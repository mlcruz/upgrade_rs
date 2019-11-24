extern crate gl;
extern crate glm;
extern crate glutin;
extern crate image;
extern crate tobj;

mod models;
mod shader;
mod world;
use glutin::dpi::LogicalSize;
use models::draw::Draw;
use models::matrix::MatrixTransform;
use models::scene_object::SceneObject;
use shader::shader_program::Shader;
use world::camera::Camera;
use world::view::View;

fn main() {
    // Variáveis que definem a câmera em coordenadas esféricas
    let g_camera_theta = 0.0; // Ângulo no plano ZX em relação ao eixo Z
    let g_camera_phi = 0.0; // Ângulo em relação ao eixo Y
    let g_camera_distance = 2.5; // Distância da câmera para a origem

    // Inicializa loop de eventos da janela
    let mut events_loop = glutin::EventsLoop::new();

    // Iniciliza janela e contexto, com perfil core, versão 3.3, tamanho 800x600
    let window = glutin::WindowBuilder::new()
        .with_title("Rust Render")
        .with_dimensions(<LogicalSize>::new(800.0f64, 600.0f64));

    let gl_window = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        .with_gl_profile(glutin::GlProfile::Core)
        .build_windowed(window, &events_loop)
        .unwrap();

    // Coloca janela no contexto atual
    let gl_window = unsafe { gl_window.make_current() }.unwrap();

    // Carrega ponteiros para funções do openGL
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    // Compila e linka shaders
    let program = Shader::new(
        "src/data/shader/vertex.glsl",
        "src/data/shader/fragment.glsl",
    )
    .program;

    // Inicializa camera
    let mut camera = Camera::new(g_camera_theta, g_camera_phi, g_camera_distance);

    // Inicializa matrizes de view e projeção com a camera criada
    let mut view = View::new(-0.01, -10.0, &camera);
    let mut is_view_orto = false;
    unsafe {
        gl::UseProgram(program);

        // Habilita Zbuffer
        gl::Enable(gl::DEPTH_TEST);

        // Inicializa uma vaca
        let cow = SceneObject::new("src/data/objs/cow.obj").scale(0.5, 0.5, 0.5);
        let bunny = SceneObject::new("src/data/objs/bunny.obj")
            .translate(1.0, 1.0, 1.0)
            .scale(0.5, 0.5, 0.5)
            .load_texture("src/data/textures/tc-earth_daymap_surface.jpg")
            .with_specular_phong_q(&16.0)
            .with_specular_reflectance(&glm::vec3(0.65, 0.5, 0.5));

        let blinking_cow = cow
            .load_texture("src/data/textures/tc-earth_nightmap_citylights.gif")
            .with_specular_reflectance(&glm::vec3(1.0, 1.0, 1.0))
            .translate(0.5, 0.5, 0.5);

        let night_cow = cow
            .load_texture("src/data/textures/tc-earth_daymap_surface.jpg")
            .with_specular_reflectance(&glm::vec3(0.8, 0.8, 0.8))
            .with_specular_phong_q(&12.0)
            .translate(-0.5, -0.5, -0.5);

        let the_horror = bunny.add_children(&blinking_cow);

        let mut should_break = false;
        loop {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);

            // Trata eventos
            events_loop.poll_events(|event| {
                use glutin::{Event, KeyboardInput, WindowEvent};
                // Limpa tela
                // Padrão é continuar o loop
                // Handling de eventos
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        // Em caso de evento de fechamento de tela, seta controle do loop de eventos para encerrar
                        WindowEvent::CloseRequested => should_break = true,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(virtual_code),
                                    state,
                                    ..
                                },
                            ..
                        } => match (virtual_code, state) {
                            (glutin::VirtualKeyCode::Up, _) => {
                                (camera.update(camera.theta, camera.phi + 0.025, camera.distance));
                            }
                            (glutin::VirtualKeyCode::Down, _) => {
                                (camera.update(camera.theta, camera.phi - 0.025, camera.distance));
                            }
                            (glutin::VirtualKeyCode::Left, _) => {
                                (camera.update(camera.theta + 0.025, camera.phi, camera.distance));
                            }
                            (glutin::VirtualKeyCode::Right, _) => {
                                (camera.update(camera.theta - 0.025, camera.phi, camera.distance));
                            }
                            (glutin::VirtualKeyCode::End, _) => {
                                (camera.update(camera.theta, camera.phi, camera.distance + 0.025));
                            }
                            (glutin::VirtualKeyCode::Home, _) => {
                                (camera.update(camera.theta, camera.phi, camera.distance - 0.025));
                            }
                            (glutin::VirtualKeyCode::O, _) => is_view_orto = true,
                            (glutin::VirtualKeyCode::P, _) => is_view_orto = false,
                            _ => (),
                        },
                        _ => (),
                    },
                    _ => (),
                }
            });

            // Atualiza possiveis modificações de camera;
            view.update_camera(&camera);

            // Prepara view
            if is_view_orto {
                view.ortographic().render(&program);
            } else {
                view.render(&program);
            }

            cow.draw(&program);
            blinking_cow.draw(&program);
            night_cow.draw(&program);
            the_horror.draw(&program);

            gl_window.swap_buffers().unwrap();

            if should_break {
                break;
            }
        }
    }
}
