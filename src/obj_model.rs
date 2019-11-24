use draw::Draw;
use draw::DrawSelf;
use gl::types::GLfloat;
use gl::types::GLsizeiptr;
use gl::types::GLuint;
use matrix::compute_normal;
use matrix::identity_matrix;
use matrix::norm;
use matrix::GLMatrix;
use matrix::MatrixTransform;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem;
use std::path::Path;
use std::ptr::null;
use tobj;

#[allow(dead_code)]
#[derive(Debug, Copy)]
pub struct ObjModel {
    pub model: GLMatrix,
    pub vao: u32,
    ebo: u32,
    geometry_vbo: u32,
    texture_vbo: u32,
    normal_vbo: u32,
    index_len: usize,
    bbox_min: glm::Vec3,
    bbox_max: glm::Vec3,
}

static ID_MATRIX: GLMatrix = identity_matrix();

#[allow(dead_code)]
impl ObjModel {
    pub fn new(path: &str) -> Self {
        // Carrega arquivo obj
        let (models, _materials) = tobj::load_obj(Path::new(path)).unwrap();

        let mut myself = ObjModel {
            vao: 0u32,
            geometry_vbo: 0u32,
            texture_vbo: 0u32,
            ebo: 0u32,
            model: ID_MATRIX,
            index_len: 0,
            normal_vbo: 0u32,
            bbox_min: glm::vec3(0.0, 0.0, 0.0),
            bbox_max: glm::vec3(0.0, 0.0, 0.0),
        };

        let mut position_array = Vec::new();
        let mut normal_array = Vec::new();
        let mut index_array = Vec::new();
        let mut texture_array = Vec::new();

        // Carrega dados de posições e indices para em vetores contínuos
        // 3 valores no vetor de indices representam os vertices de um indice
        // 4 valores no vetor de posição representam a posição de um vertice
        for (_index, model) in models.iter().enumerate() {
            let mesh = &model.mesh;

            for f in 0..mesh.indices.len() {
                // Vertices X Y Z de um triangulo
                index_array.push(mesh.indices[f]);
            }

            for v in 0..mesh.positions.len() / 3 {
                // Insere uma posição de um vertice
                // X Y Z W em ordem
                let x = mesh.positions[3 * v];
                let y = mesh.positions[3 * v + 1];
                let z = mesh.positions[3 * v + 2];

                position_array.push(x);
                position_array.push(y);
                position_array.push(z);
                position_array.push(1f32);

                myself.bbox_min.x = glm::min(myself.bbox_min.x, x);
                myself.bbox_min.y = glm::min(myself.bbox_min.y, y);
                myself.bbox_min.z = glm::min(myself.bbox_min.z, z);
                myself.bbox_max.x = glm::max(myself.bbox_max.x, x);
                myself.bbox_max.y = glm::max(myself.bbox_max.y, y);
                myself.bbox_max.z = glm::max(myself.bbox_max.z, z);
            }

            // Insere texturas
            if mesh.texcoords.len() > 0 {
                for v in 0..mesh.texcoords.len() {
                    texture_array.push(mesh.texcoords[v]);
                }
            }

            // Verifica se existem normais no obj, e insere
            if mesh.normals.len() > 0 {
                for v in 0..mesh.normals.len() / 3 {
                    // Insere um normal de um vertice
                    // X Y Z W em ordem
                    normal_array.push(mesh.positions[3 * v]);
                    normal_array.push(mesh.positions[3 * v + 1]);
                    normal_array.push(mesh.positions[3 * v + 2]);
                    normal_array.push(0f32);
                }
            } else {
                // Computa normais dos vertices se não existe no obj
                let num_vertices = mesh.positions.len() / 3;

                let mut num_triangles_per_vertex: Vec<u32> = vec![0; num_vertices];

                let mut vertex_normals: Vec<glm::Vec4> =
                    vec![glm::Vec4::new(0f32, 0f32, 0f32, 1f32); num_vertices];

                //println!("{:?}", mesh.indices.len() / 3);
                for t in 0..mesh.indices.len() / 3 {
                    // Calcula a normal de todos os triangulos

                    let mut triangle_vertex_array = [
                        glm::Vec4::new(0f32, 0f32, 0f32, 1f32),
                        glm::Vec4::new(0f32, 0f32, 0f32, 1f32),
                        glm::Vec4::new(0f32, 0f32, 0f32, 1f32),
                    ];

                    for v in 0..3 {
                        let idx = mesh.indices[3 * t + v] as usize;
                        let vx = mesh.positions[3 * idx + 0];
                        let vy = mesh.positions[3 * idx + 1];
                        let vz = mesh.positions[3 * idx + 2];

                        triangle_vertex_array[v].x = vx;
                        triangle_vertex_array[v].y = vy;
                        triangle_vertex_array[v].z = vz;
                    }

                    let [a, b, c] = triangle_vertex_array;
                    let n = compute_normal(&a, &b, &c);
                    for v in 0..3 {
                        let idx = mesh.indices[3 * t + v] as usize;
                        num_triangles_per_vertex[idx] = num_triangles_per_vertex[idx] + 1;
                        vertex_normals[idx] = vertex_normals[idx] + n;
                    }
                }

                for i in 0..vertex_normals.len() {
                    let mut n = vertex_normals[i] / num_triangles_per_vertex[i] as f32;
                    n = n / norm(n);

                    normal_array.push(n.x);
                    normal_array.push(n.y);
                    normal_array.push(n.z);
                    normal_array.push(0f32);
                }
            }
        }

        // Alocação de VAO e VBOS
        unsafe {
            // Definição dos atributos dos vertices
            // Cria VAO do obj e "liga" ele
            gl::GenVertexArrays(1, &mut myself.vao);
            gl::BindVertexArray(myself.vao);

            // Cria identificador do VBO a ser utilizado pelos atributos de geometria e "liga" o mesmo
            gl::GenBuffers(1, &mut myself.geometry_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, myself.geometry_vbo);

            // Aloca memória para o VBO acima.
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (position_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, // Tamanho dos vertices
                null(),
                gl::STATIC_DRAW,
            );

            //Copia valores dos array de vertices para o VBO
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (position_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                position_array.as_ptr() as *const c_void,
            );

            // Location no shader para o VBO acima
            let location: GLuint = 0; // location 0 no vertex shader

            // "Liga" VAO e VBO
            gl::VertexAttribPointer(location, 4, gl::FLOAT, gl::FALSE, 0, null());
            // Ativa atributos
            gl::EnableVertexAttribArray(location);
            // Desliga VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            //Normais:
            if normal_array.len() > 0 {
                gl::GenBuffers(1, &mut myself.normal_vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, myself.normal_vbo);

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (normal_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    null(),
                    gl::STATIC_DRAW,
                );
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (normal_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    normal_array.as_ptr() as *const c_void,
                );

                // Location das normais no shader
                let location: GLuint = 1;

                gl::VertexAttribPointer(location, 4, gl::FLOAT, gl::FALSE, 0, null());
                gl::EnableVertexAttribArray(location);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }

            // Aloca texturas
            if texture_array.len() > 0 {
                gl::GenBuffers(1, &mut myself.texture_vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, myself.texture_vbo);

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (texture_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    null(),
                    gl::STATIC_DRAW,
                );
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (texture_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    texture_array.as_ptr() as *const c_void,
                );

                // Location das texturas no shader
                let location: GLuint = 2;

                gl::VertexAttribPointer(location, 4, gl::FLOAT, gl::FALSE, 0, null());
                gl::EnableVertexAttribArray(location);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }

            // Topolgia:
            gl::GenBuffers(1, &mut myself.ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, myself.ebo);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (index_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                null(),
                gl::STATIC_DRAW,
            );
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                0,
                (index_array.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                index_array.as_ptr() as *const c_void,
            );
            gl::BindVertexArray(0);
            myself.index_len = index_array.len();
        }
        myself
    }
}

impl MatrixTransform for ObjModel {
    fn get_matrix(&self) -> &GLMatrix {
        &self.model
    }
    fn update_matrix(&mut self, matrix: &GLMatrix) -> &Self {
        self.model = matrix.clone();
        self
    }
    fn from_matrix(&self, matrix: &GLMatrix) -> Self {
        *self.clone().update_matrix(matrix)
    }
}

impl Clone for ObjModel {
    fn clone(&self) -> Self {
        *self
    }
}

impl DrawSelf for ObjModel {
    fn draw_self(&self, program: &u32) -> &Self {
        unsafe {
            gl::UseProgram(*program);

            gl::BindVertexArray(self.vao);

            let model_uniform =
                gl::GetUniformLocation(*program, CString::new("model").unwrap().as_ptr());

            gl::UniformMatrix4fv(
                model_uniform,
                1,
                gl::FALSE,
                mem::transmute(&self.model.matrix[0]),
            );

            gl::DrawElements(
                gl::TRIANGLES,
                self.index_len as i32,
                gl::UNSIGNED_INT,
                0 as *const i32 as *const c_void,
            );
        }
        self
    }
}

impl Draw for ObjModel {
    fn draw(&self, program: &u32) -> &Self {
        unsafe {
            gl::UseProgram(*program);

            gl::BindVertexArray(self.vao);

            let model_uniform =
                gl::GetUniformLocation(*program, CString::new("model").unwrap().as_ptr());

            let bbox_min_uniform =
                gl::GetUniformLocation(*program, CString::new("bbox_min").unwrap().as_ptr());
            let bbox_max_uniform =
                gl::GetUniformLocation(*program, CString::new("bbox_max").unwrap().as_ptr());

            // Setamos as variáveis "bbox_min" e "bbox_max" do fragment shader
            // com os parâmetros da axis-aligned bounding box (AABB) do modelo.
            gl::Uniform4f(
                bbox_min_uniform,
                self.bbox_min.x,
                self.bbox_min.y,
                self.bbox_min.z,
                1.0,
            );

            gl::Uniform4f(
                bbox_max_uniform,
                self.bbox_max.x,
                self.bbox_max.y,
                self.bbox_max.z,
                1.0,
            );

            gl::UniformMatrix4fv(
                model_uniform,
                1,
                gl::FALSE,
                mem::transmute(&self.model.matrix[0]),
            );

            gl::DrawElements(
                gl::TRIANGLES,
                self.index_len as i32,
                gl::UNSIGNED_INT,
                0 as *const i32 as *const c_void,
            );
        }
        self
    }
}
