use anyhow::Result;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
use gltf::material::AlphaMode;
use image::DynamicImage;

use crate::{
    material::Material,
    mesh::{generate_normals, generate_tangents, pack_vertices, Mesh},
    texture::{Texture, TextureCreateDesc, TextureFormat, UncompressedTextureFormat},
    Model, ModelNode,
};

use super::ParseOptions;

pub(crate) fn parse_glb(data: &[u8], opt: ParseOptions) -> Result<Model> {
    let (document, buffers, images) = gltf::import_slice(data)?;

    let mut meshes = vec![None; document.meshes().len()];
    let mut materials = vec![Material::default(); document.materials().len()];
    let mut textures = vec![];

    let mut image_to_texture_mapping = vec![None; document.images().len()];

    if materials.is_empty() {
        materials.push(Material::default());
    }

    let mut root_node_indices = Vec::new();
    let mut nodes = Vec::new();

    if let Some(scene) = document.default_scene() {
        for root_node in scene.nodes() {
            root_node_indices.push(nodes.len() as u32);
            process_nodes_recursive(
                &document,
                &root_node,
                &buffers,
                &images,
                &mut nodes,
                &mut textures,
                &mut image_to_texture_mapping,
                &mut materials,
                &mut meshes,
                opt,
            );
        }
    }

    let meshes = meshes.into_iter().map(|mesh| mesh.unwrap()).collect();

    Ok(Model {
        root_node_indices,
        nodes,

        meshes,
        materials,
        textures,
    })
}

#[allow(clippy::too_many_arguments)]
fn process_nodes_recursive(
    document: &gltf::Document,
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    nodes: &mut Vec<ModelNode>,
    internal_images: &mut Vec<Texture>,
    image_to_texture_mapping: &mut [Option<u32>],
    materials: &mut Vec<Material>,
    meshes: &mut Vec<Option<Mesh>>,
    opt: ParseOptions,
) {
    nodes.push(process_node(
        document,
        node,
        buffers,
        images,
        internal_images,
        image_to_texture_mapping,
        materials,
        meshes,
        opt,
    ));
    let node_idx = nodes.len() - 1;

    for child in node.children() {
        let child_idx = nodes.len() as u32;
        nodes[node_idx].child_node_indices.push(child_idx);
        process_nodes_recursive(
            document,
            &child,
            buffers,
            images,
            nodes,
            internal_images,
            image_to_texture_mapping,
            materials,
            meshes,
            opt,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn process_node(
    document: &gltf::Document,
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    internal_images: &mut Vec<Texture>,
    image_to_texture_mapping: &mut [Option<u32>],
    materials: &mut [Material],
    meshes: &mut [Option<Mesh>],
    opt: ParseOptions,
) -> ModelNode {
    let (translation, rotation, scale) = node.transform().decomposed();
    let translation = Vec3::new(translation[0], translation[1], translation[2]);
    let rotation = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);
    let scale = Vec3::new(scale[0], scale[1], scale[2]);
    let transform =
        Mat4::from_scale_rotation_translation(scale, rotation, translation).to_cols_array();

    let mut node_mesh = None;

    if let Some(mesh) = node.mesh() {
        let mesh_idx = mesh.index();
        if meshes[mesh_idx].is_none() {
            let mut mesh_vertex_positions = vec![];
            let mut mesh_vertex_tex_coords = vec![];
            let mut mesh_vertex_normals = vec![];
            let mut mesh_vertex_tangents = vec![];
            let mut mesh_triangle_material_indices = vec![];
            let mut mesh_indices = vec![];
            let mut opaque = true;
            let mut is_emissive = false;

            for primitive in mesh.primitives() {
                if primitive.mode() == gltf::mesh::Mode::Triangles {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    let mut vertex_positions = {
                        let iter = reader
                            .read_positions()
                            .expect("Failed to process mesh node. (Vertices must have positions)");

                        iter.map(|arr| -> Vec3 { Vec3::from(arr) })
                            .collect::<Vec<_>>()
                    };

                    let indices = reader
                        .read_indices()
                        .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>())
                        .expect("Failed to process mesh node. (Indices are required)");

                    let mut vertex_tex_coords = if let Some(tex_coords) = reader.read_tex_coords(0)
                    {
                        tex_coords
                            .into_f32()
                            .map(|tex_coord| -> Vec2 { Vec2::from(tex_coord) })
                            .collect()
                    } else {
                        vec![]
                    };

                    let mut vertex_normals = if let Some(normals) = reader.read_normals() {
                        normals
                            .into_iter()
                            .map(|normal| -> Vec3 { Vec3::from(normal) })
                            .collect()
                    } else {
                        vec![]
                    };

                    let mut vertex_tangents = if let Some(tangents) = reader.read_tangents() {
                        tangents
                            .into_iter()
                            .map(|tangent| -> Vec4 { Vec4::from(tangent) })
                            .collect()
                    } else {
                        vec![]
                    };

                    let num_triangles = indices.len() / 3;

                    let mut indices = indices
                        .into_iter()
                        .map(|index| index + mesh_vertex_positions.len() as u32)
                        .collect::<Vec<u32>>();
                    mesh_vertex_positions.append(&mut vertex_positions);
                    mesh_vertex_tex_coords.append(&mut vertex_tex_coords);
                    mesh_vertex_normals.append(&mut vertex_normals);
                    mesh_vertex_tangents.append(&mut vertex_tangents);
                    mesh_indices.append(&mut indices);

                    let prim_material = primitive.material();
                    let pbr = prim_material.pbr_metallic_roughness();
                    let material_idx = primitive.material().index().unwrap_or(0);

                    mesh_triangle_material_indices
                        .append(&mut vec![material_idx as u32; num_triangles]);

                    let material = &mut materials[material_idx];
                    if material.index.is_none() {
                        material.index = Some(material_idx);
                        material.name = prim_material.name().unwrap_or("Unnamed").to_owned();

                        material.color = Vec4::from(pbr.base_color_factor()).xyz().to_array();
                        material.metallic = pbr.metallic_factor();
                        material.roughness = pbr.roughness_factor();
                        material.emission = (Vec3::from(prim_material.emissive_factor())
                            * prim_material.emissive_strength().unwrap_or(1.0))
                        .to_array();

                        if let Some(volume) = prim_material.volume() {
                            // TODO: not 100 percent sure this is correct
                            material.absorption = ((Vec3::ONE
                                - Vec3::from(volume.attenuation_color()))
                                / volume.attenuation_distance())
                            .to_array();
                        }
                        if let Some(transmission) = prim_material.transmission() {
                            material.transmission = transmission.transmission_factor();
                            if let Some(tex) = transmission.transmission_texture() {
                                material.transmission_texture = Some(process_tex_info(
                                    document,
                                    images,
                                    internal_images,
                                    image_to_texture_mapping,
                                    &tex,
                                    opt,
                                ));
                            }
                        }
                        material.eta = 1.0 / prim_material.ior().unwrap_or(1.5);

                        material.subsurface = 0.0; // TODO
                        if let Some(specular) = prim_material.specular() {
                            material.specular = specular.specular_factor();
                            material.specular_tint = specular.specular_color_factor();
                        }
                        // Pending PR at gltf-rs: https://github.com/gltf-rs/gltf/pull/446
                        if let Some(clearcoat) = prim_material.clearcoat() {
                            material.clearcoat = clearcoat.clearcoat_factor();
                            if let Some(tex) = clearcoat.clearcoat_texture() {
                                material.clearcoat_texture = Some(process_tex_info(
                                    document,
                                    images,
                                    internal_images,
                                    image_to_texture_mapping,
                                    &tex,
                                    opt,
                                ));
                            }
                            material.clearcoat_roughness = clearcoat.clearcoat_roughness_factor();
                            if let Some(tex) = clearcoat.clearcoat_roughness_texture() {
                                material.clearcoat_roughness_texture = Some(process_tex_info(
                                    document,
                                    images,
                                    internal_images,
                                    image_to_texture_mapping,
                                    &tex,
                                    opt,
                                ));
                            }
                            if let Some(tex) = clearcoat.clearcoat_normal_texture() {
                                material.clearcoat_normal_texture = Some(process_normal_tex(
                                    document,
                                    images,
                                    internal_images,
                                    image_to_texture_mapping,
                                    tex,
                                    opt,
                                ));
                            }
                        }
                        if let Some(sheen) = prim_material.sheen() {
                            material.sheen = sheen.sheen_roughness_factor();
                            if let Some(tex) = sheen.sheen_roughness_texture() {
                                material.sheen_texture = Some(process_tex_info(
                                    document,
                                    images,
                                    internal_images,
                                    image_to_texture_mapping,
                                    &tex,
                                    opt,
                                ));
                            }
                            material.sheen_tint = sheen.sheen_color_factor();
                            if let Some(tex) = sheen.sheen_color_texture() {
                                material.sheen_tint_texture = Some(process_tex_info(
                                    document,
                                    images,
                                    internal_images,
                                    image_to_texture_mapping,
                                    &tex,
                                    opt,
                                ));
                            }
                        }

                        material.alpha_cutoff = prim_material.alpha_cutoff().unwrap_or(0.5);
                        material.is_opaque = prim_material.alpha_mode() == AlphaMode::Opaque
                            || material.alpha_cutoff == 0.0;

                        if let Some(tex) = pbr.base_color_texture() {
                            material.color_texture = Some(process_tex_info(
                                document,
                                images,
                                internal_images,
                                image_to_texture_mapping,
                                &tex,
                                opt,
                            ));
                        }

                        if let Some(tex) = prim_material.normal_texture() {
                            material.normal_scale = tex.scale();
                            material.normal_texture = Some(process_normal_tex(
                                document,
                                images,
                                internal_images,
                                image_to_texture_mapping,
                                tex,
                                opt,
                            ));
                        }

                        if let Some(tex) = pbr.metallic_roughness_texture() {
                            material.metallic_roughness_texture = Some(process_tex_info(
                                document,
                                images,
                                internal_images,
                                image_to_texture_mapping,
                                &tex,
                                opt,
                            ));
                        }

                        if let Some(tex) = prim_material.emissive_texture() {
                            material.emission_texture = Some(process_tex_info(
                                document,
                                images,
                                internal_images,
                                image_to_texture_mapping,
                                &tex,
                                opt,
                            ));
                        }
                    }

                    opaque = opaque && material.is_opaque;
                    is_emissive = is_emissive || material.is_emissive();
                } else {
                    panic!("Only triangles are supported.");
                }
            }

            if mesh_vertex_normals.is_empty() {
                mesh_vertex_normals = generate_normals(&mesh_vertex_positions, &mesh_indices);
            }
            if mesh_vertex_tangents.is_empty() {
                mesh_vertex_tangents = generate_tangents(
                    &mesh_vertex_positions,
                    &mesh_vertex_normals,
                    &mesh_vertex_tex_coords,
                    &mesh_indices,
                );
            }
            if mesh_vertex_tex_coords.is_empty() {
                mesh_vertex_tex_coords = vec![Vec2::ZERO; mesh_vertex_positions.len()];
            }

            let packed_vertices = pack_vertices(
                mesh_vertex_positions,
                mesh_vertex_normals,
                mesh_vertex_tangents,
                mesh_vertex_tex_coords,
            );

            let mesh = Mesh::new(
                packed_vertices,
                mesh_triangle_material_indices,
                mesh_indices,
                opaque,
                is_emissive,
            );

            meshes[mesh_idx] = Some(mesh);
        }

        node_mesh = Some(mesh_idx as u32);
    }

    ModelNode {
        name: node.name().unwrap_or("Unnamed").to_owned(),
        transform,
        mesh_idx: node_mesh,
        child_node_indices: vec![],
    }
}

fn process_tex_info(
    document: &gltf::Document,
    images: &[gltf::image::Data],
    internal_images: &mut Vec<Texture>,
    image_to_texture_mapping: &mut [Option<u32>],
    texture_info: &gltf::texture::Info,
    opt: ParseOptions,
) -> u32 {
    let texture = texture_info.texture();
    let texture_transform = texture_info.texture_transform();

    process_tex(
        document,
        images,
        internal_images,
        image_to_texture_mapping,
        texture,
        texture_transform,
        opt,
    )
}

fn process_normal_tex(
    document: &gltf::Document,
    images: &[gltf::image::Data],
    internal_images: &mut Vec<Texture>,
    image_to_texture_mapping: &mut [Option<u32>],
    normal_tex: gltf::material::NormalTexture,
    opt: ParseOptions,
) -> u32 {
    let texture = normal_tex.texture();
    let texture_transform = normal_tex.texture_transform();

    process_tex(
        document,
        images,
        internal_images,
        image_to_texture_mapping,
        texture,
        texture_transform,
        opt,
    )
}

fn process_tex(
    document: &gltf::Document,
    images: &[gltf::image::Data],
    internal_images: &mut Vec<Texture>,
    image_to_texture_mapping: &mut [Option<u32>],
    texture: gltf::texture::Texture,
    texture_transform: Option<gltf::texture::TextureTransform>,
    opt: ParseOptions,
) -> u32 {
    let name = texture.name().unwrap_or("Unnamed");

    let (uv_offset, uv_scale) = if let Some(transform) = texture_transform {
        (transform.offset(), transform.scale())
    } else {
        ([0.0; 2], [1.0; 2])
    };

    match texture.source().source() {
        gltf::image::Source::View { .. } => {
            let texture_idx = texture.index(); // TODO???

            let texture = document.textures().nth(texture_idx).unwrap(); // TODO ???
            let image_idx = texture.source().index();

            if let Some(texture_idx) = &image_to_texture_mapping[image_idx] {
                *texture_idx
            } else {
                let data = images[image_idx].clone();

                let create_desc = match data.format {
                    gltf::image::Format::R8G8B8 => {
                        let dynamic_image = DynamicImage::ImageRgb8(
                            image::RgbImage::from_raw(data.width, data.height, data.pixels)
                                .unwrap(),
                        );
                        let image = dynamic_image.to_rgba8();

                        TextureCreateDesc {
                            name: Some(name),
                            width: data.width,
                            height: data.height,
                            format: TextureFormat::Uncompressed(
                                UncompressedTextureFormat::Rgba8Unorm,
                            ),
                            data: image.as_raw().clone(),
                            uv_offset,
                            uv_scale,
                        }
                    }
                    gltf::image::Format::R16G16B16 => {
                        let mut u8_pixels = vec![];
                        for y in 0..data.width {
                            for x in 0..data.height {
                                for c in 0..3 {
                                    let i = ((y * data.width + x) * 3 + c) as usize;
                                    let u16_value = u16::from_le_bytes([
                                        data.pixels[i * 2],
                                        data.pixels[i * 2 + 1],
                                    ]);
                                    u8_pixels.push((u16_value / 257) as u8);
                                }
                            }
                        }

                        let dynamic_image = DynamicImage::ImageRgb8(
                            image::RgbImage::from_raw(data.width, data.height, u8_pixels).unwrap(),
                        );
                        let image = dynamic_image.to_rgba8();

                        TextureCreateDesc {
                            name: Some(name),
                            width: data.width,
                            height: data.height,
                            format: TextureFormat::Uncompressed(
                                UncompressedTextureFormat::Rgba8Unorm,
                            ),
                            data: image.as_raw().clone(),
                            uv_offset,
                            uv_scale,
                        }
                    }
                    _ => {
                        let format = match data.format {
                            gltf::image::Format::R8G8B8A8 => {
                                TextureFormat::Uncompressed(UncompressedTextureFormat::Rgba8Unorm)
                            }
                            gltf::image::Format::R8G8 => {
                                TextureFormat::Uncompressed(UncompressedTextureFormat::Rg8Unorm)
                            }
                            gltf::image::Format::R8 => {
                                TextureFormat::Uncompressed(UncompressedTextureFormat::R8Unorm)
                            }
                            _ => panic!("Unsupported image type: {:?}.", data.format),
                        };

                        TextureCreateDesc {
                            name: Some(name),
                            width: data.width,
                            height: data.height,
                            format,
                            data: data.pixels,
                            uv_offset,
                            uv_scale,
                        }
                    }
                };

                // let create_desc = if data.format == gltf::image::Format::R8G8B8 {
                //     let dynamic_image = DynamicImage::ImageRgb8(
                //         image::RgbImage::from_raw(data.width, data.height, data.pixels).unwrap(),
                //     );
                //     let image = dynamic_image.to_rgba8();

                //     TextureCreateDesc {
                //         name: Some(name),
                //         width: data.width,
                //         height: data.height,
                //         format: TextureFormat::Uncompressed(UncompressedTextureFormat::Rgba8Unorm),
                //         data: image.as_raw().clone(),
                //     }
                // } else {
                //     let format = match data.format {
                //         gltf::image::Format::R8G8B8A8 => {
                //             TextureFormat::Uncompressed(UncompressedTextureFormat::Rgba8Unorm)
                //         }
                //         gltf::image::Format::R8G8 => {
                //             TextureFormat::Uncompressed(UncompressedTextureFormat::Rg8Unorm)
                //         }
                //         gltf::image::Format::R8 => {
                //             TextureFormat::Uncompressed(UncompressedTextureFormat::R8Unorm)
                //         }
                //         _ => panic!("Unsupported image type: {:?}.", data.format),
                //     };

                //     TextureCreateDesc {
                //         name: Some(name),
                //         width: data.width,
                //         height: data.height,
                //         format,
                //         data: data.pixels,
                //     }
                // };

                let mut texture = Texture::new(create_desc);
                if let Some(texture_compression) = &opt.texture_compression {
                    if let Some(compressed_texture) = texture.compress(texture_compression) {
                        texture = compressed_texture;
                    }
                }

                let texture_idx = internal_images.len() as u32;
                internal_images.push(texture);
                image_to_texture_mapping[image_idx] = Some(texture_idx);
                texture_idx
            }
        }
        gltf::image::Source::Uri { .. } => todo!(),
    }
}
