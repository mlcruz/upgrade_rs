use super::complex_obj::ComplexObj;
use super::composite_obj::CompositeObj;
use super::draw::Draw;
use super::matrix::GLMatrix;
use super::matrix::MatrixTransform;
use super::obj_model::ObjModel;
use models::load_texture::load_texture;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum SceneObject {
    CompositeObj(CompositeObj),
    ObjModel(ObjModel),
    ComplexObj(ComplexObj),
}

#[allow(dead_code)]
impl SceneObject {
    pub fn new(path: &str) -> Self {
        SceneObject::ObjModel(ObjModel::new(path))
    }

    pub fn add_children(&self, child: &SceneObject) -> Self {
        match self {
            SceneObject::ObjModel(obj_model) => match child {
                SceneObject::ObjModel(c_obj) => SceneObject::CompositeObj(CompositeObj {
                    root: *obj_model,
                    children: vec![*c_obj],
                }),
                SceneObject::CompositeObj(c_cmp) => SceneObject::ComplexObj(ComplexObj {
                    root: *obj_model,
                    children: Box::new(vec![SceneObject::CompositeObj(c_cmp.clone())]),
                }),
                SceneObject::ComplexObj(c_cplx) => SceneObject::ComplexObj(ComplexObj {
                    root: *obj_model,
                    children: Box::new(vec![SceneObject::ComplexObj(c_cplx.clone())]),
                }),
            },
            SceneObject::CompositeObj(cmp_model) => match child {
                SceneObject::ObjModel(c_obj) => {
                    SceneObject::CompositeObj(cmp_model.add_children(c_obj))
                }
                SceneObject::CompositeObj(c_cmp) => {
                    let mut new_children: Vec<SceneObject> = cmp_model
                        .children
                        .clone()
                        .iter()
                        .map(|item| SceneObject::ObjModel(*item))
                        .collect();

                    new_children.append(&mut vec![SceneObject::CompositeObj((*c_cmp).clone())]);

                    SceneObject::ComplexObj(ComplexObj {
                        root: cmp_model.root,
                        children: Box::new(new_children),
                    })
                }
                SceneObject::ComplexObj(c_cplx) => {
                    let mut new_children: Vec<SceneObject> = cmp_model
                        .children
                        .clone()
                        .iter()
                        .map(|item| SceneObject::ObjModel(*item))
                        .collect();

                    new_children.append(&mut vec![SceneObject::ComplexObj((*c_cplx).clone())]);

                    SceneObject::ComplexObj(ComplexObj {
                        root: cmp_model.root,
                        children: Box::new(new_children),
                    })
                }
            },
            SceneObject::ComplexObj(cplx_model) => {
                SceneObject::ComplexObj(cplx_model.add_children(child))
            }
        }
    }

    pub fn get_bbox_min(&self) -> glm::Vec3 {
        match self {
            SceneObject::ObjModel(obj) => obj.bbox_min,
            SceneObject::CompositeObj(obj) => obj.root.bbox_min,
            SceneObject::ComplexObj(obj) => obj.root.bbox_min,
        }
    }

    pub fn get_bbox_max(&self) -> glm::Vec3 {
        match self {
            SceneObject::ObjModel(obj) => obj.bbox_max,
            SceneObject::CompositeObj(obj) => obj.root.bbox_max,
            SceneObject::ComplexObj(obj) => obj.root.bbox_max,
        }
    }
    #[allow(dead_code)]
    pub fn with_texture(&self, texture: &u32) -> Self {
        match self {
            SceneObject::ObjModel(obj) => SceneObject::ObjModel(obj.with_texture(texture)),
            SceneObject::CompositeObj(obj) => SceneObject::CompositeObj(CompositeObj {
                root: obj.root.with_texture(texture),
                children: obj.children.clone(),
            }),
            SceneObject::ComplexObj(obj) => SceneObject::ComplexObj(ComplexObj {
                root: obj.root.with_texture(texture),
                children: obj.children.clone(),
            }),
        }
    }

    pub fn with_specular_reflectance(&self, specular_reflectance: &glm::Vec3) -> Self {
        match self {
            SceneObject::ObjModel(obj) => {
                SceneObject::ObjModel(obj.with_specular_reflectance(specular_reflectance))
            }
            SceneObject::CompositeObj(obj) => SceneObject::CompositeObj(CompositeObj {
                root: obj.root.with_specular_reflectance(specular_reflectance),
                children: obj.children.clone(),
            }),
            SceneObject::ComplexObj(obj) => SceneObject::ComplexObj(ComplexObj {
                root: obj.root.with_specular_reflectance(specular_reflectance),
                children: obj.children.clone(),
            }),
        }
    }

    pub fn with_specular_phong_q(&self, phong_q: &f32) -> Self {
        match self {
            SceneObject::ObjModel(obj) => SceneObject::ObjModel(obj.with_specular_phong_q(phong_q)),
            SceneObject::CompositeObj(obj) => SceneObject::CompositeObj(CompositeObj {
                root: obj.root.with_specular_phong_q(phong_q),
                children: obj.children.clone(),
            }),
            SceneObject::ComplexObj(obj) => SceneObject::ComplexObj(ComplexObj {
                root: obj.root.with_specular_phong_q(phong_q),
                children: obj.children.clone(),
            }),
        }
    }

    pub fn with_color(&self, color: &glm::Vec3) -> Self {
        match self {
            SceneObject::ObjModel(obj) => SceneObject::ObjModel(obj.with_color(color)),
            SceneObject::CompositeObj(obj) => SceneObject::CompositeObj(CompositeObj {
                root: obj.root.with_color(color),
                children: obj.children.clone(),
            }),
            SceneObject::ComplexObj(obj) => SceneObject::ComplexObj(ComplexObj {
                root: obj.root.with_color(color),
                children: obj.children.clone(),
            }),
        }
    }
    pub unsafe fn load_texture(&self, path: &str) -> Self {
        let (tex, _) = load_texture(path);
        self.with_texture(&tex)
    }

    pub fn check_intersection(&self, obj2: &SceneObject) -> bool {
        //let model_translation = obj1.model.matrix.c3;
        let obj1 = self;

        // Utiliza transação do obj para calcular pos global
        let obj1_t = obj1.get_matrix().matrix.c3;
        let obj2_t = obj2.get_matrix().matrix.c3;
        let obj1_bbox_min = obj1.get_bbox_min();
        let obj1_bbox_max = obj1.get_bbox_max();
        let obj2_bbox_min = obj2.get_bbox_min();
        let obj2_bbox_max = obj2.get_bbox_max();
        // Pos global da bbox  do obj1
        let obj1_bbox_min_pos = obj1.get_matrix().matrix
            * glm::vec4(obj1_bbox_min.x, obj1_bbox_min.y, obj1_bbox_min.z, 0.0)
            + obj1_t;
        let obj1_bbox_max_pos = obj1.get_matrix().matrix
            * glm::vec4(obj1_bbox_max.x, obj1_bbox_max.y, obj1_bbox_max.z, 0.0)
            + obj1_t;
        // Pos global da bbox  do obj2
        let obj2_bbox_min_pos = obj1.get_matrix().matrix
            * glm::vec4(obj2_bbox_min.x, obj2_bbox_min.y, obj2_bbox_min.z, 0.0)
            + obj2_t;
        let obj2_bbox_max_pos = obj1.get_matrix().matrix
            * glm::vec4(obj2_bbox_max.x, obj2_bbox_max.y, obj2_bbox_max.z, 0.0)
            + obj2_t;
        check_bbox_bbox_intersection(
            &obj1_bbox_min_pos,
            &obj1_bbox_max_pos,
            &obj2_bbox_min_pos,
            &obj2_bbox_max_pos,
        )
    }
}

#[allow(dead_code)]
impl Draw for SceneObject {
    fn draw(&self, program: &u32) -> &Self {
        match self {
            SceneObject::ObjModel(obj_model) => {
                obj_model.draw(&program);
            }
            SceneObject::ComplexObj(complex_obj) => {
                complex_obj.draw(&program);
            }
            SceneObject::CompositeObj(composite_obj) => {
                composite_obj.draw(&program);
            }
        }
        self
    }
}

impl MatrixTransform for SceneObject {
    fn get_matrix(&self) -> &GLMatrix {
        match self {
            SceneObject::ObjModel(obj_model) => &obj_model.get_matrix(),
            SceneObject::ComplexObj(complex_obj) => &complex_obj.get_matrix(),
            SceneObject::CompositeObj(composite_obj) => &composite_obj.get_matrix(),
        }
    }

    fn update_matrix(&mut self, matrix: &GLMatrix) -> &mut Self {
        match self {
            SceneObject::ObjModel(obj_model) => {
                let new_me = obj_model.update_matrix(matrix).clone();
                *self = SceneObject::ObjModel(new_me);
                self
            }
            SceneObject::ComplexObj(complex_obj) => {
                *self = SceneObject::ComplexObj(complex_obj.update_matrix(matrix).clone());
                self
            }
            SceneObject::CompositeObj(composite_obj) => {
                *self = SceneObject::CompositeObj(composite_obj.update_matrix(matrix).clone());
                self
            }
        }
    }
    fn from_matrix(&self, matrix: &GLMatrix) -> Self {
        match self {
            SceneObject::ObjModel(obj_model) => {
                SceneObject::ObjModel(obj_model.from_matrix(matrix))
            }
            SceneObject::ComplexObj(complex_obj) => {
                SceneObject::ComplexObj(complex_obj.from_matrix(matrix))
            }
            SceneObject::CompositeObj(composite_obj) => {
                SceneObject::CompositeObj(composite_obj.from_matrix(matrix))
            }
        }
    }
}

#[allow(dead_code)]
pub fn check_point_bbox_intersection(
    bbox_min: &glm::Vec4,
    bbox_max: &glm::Vec4,
    point: &glm::Vec4,
) -> bool {
    return (point.x >= bbox_min.x && point.x <= bbox_max.x)
        && (point.y >= bbox_min.y && point.y <= bbox_max.y)
        && (point.z >= bbox_min.z && point.z <= bbox_max.z);
}

#[allow(dead_code)]
pub fn check_bbox_bbox_intersection(
    bbox1_min: &glm::Vec4,
    bbox1_max: &glm::Vec4,
    bbox2_min: &glm::Vec4,
    bbox2_max: &glm::Vec4,
) -> bool {
    return (bbox1_min.x <= bbox2_max.x && bbox1_max.x >= bbox2_min.x)
        && (bbox1_min.y <= bbox2_max.y && bbox1_max.y >= bbox2_min.y)
        && (bbox1_min.z <= bbox2_max.z && bbox1_max.z >= bbox2_min.z);
}
