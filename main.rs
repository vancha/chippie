/// I think this encapsulates the "widget" 
mod chip8emulator {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::border;
    use iced::mouse;
    use iced::{Color, Element, Length, Rectangle, Size};
    use chippie_emulator::cpu::Cpu;
     use chippie_emulator::rombuffer::RomBuffer;
    
    
    //The preffered size of the blocks on screen
    const BLOCK_SIZE: f32 = 10.0;
    //this emulatorwrapper struct holds (or owns?) a chip8 emulator instance
    pub struct EmulatorWrapper {
        emulator: Option<Cpu>,
    }

    impl EmulatorWrapper {
        pub fn new(path: &str) -> Self {

            Self {
                emulator:Some(Cpu::new(&RomBuffer::new(path))),
            }
        }
    }

    pub fn emulator(path: &str) -> EmulatorWrapper {
        EmulatorWrapper::new(path)
    }

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for EmulatorWrapper
    where
        Renderer: renderer::Renderer,
    {
        fn size(&self) -> Size<Length> {
            Size {
                width: Length::Shrink,
                height: Length::Shrink,
            }
        }

        fn layout(
            &mut self,
            _tree: &mut widget::Tree,
            _renderer: &Renderer,
            //This tells us what size contraints we have to abide by
            limits: &layout::Limits,
        ) -> layout::Node {
            // maintain our aspect ratio, attempting to draw at preferred block size
            let preferred_width = 64.0 * BLOCK_SIZE;
            let preferred_height = 32.0 * BLOCK_SIZE;

            let max_width = limits.max().width;
            let max_height = limits.max().height;

            // Compute the scale needed to fit inside the max bounds.
            // If preferred size is already small enough, scale = 1.0.
            let scale_width = max_width / preferred_width;
            let scale_height = max_height / preferred_height;

            // Use the smaller scale to ensure BOTH dimensions fit.
            let scale = scale_width.min(scale_height).min(1.0);

            // Apply the scale to both dimensions.
            let width = preferred_width * scale;
            let height = preferred_height * scale;

            layout::Node::new(Size::new(width, height))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            match &self.emulator {
                Some(emulator) => {
                    println!("I actually have an emulator to draw");
                }
                None => {
                    println!("I have nothing to draw");
                }
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..renderer::Quad::default()
                },
                Color::WHITE,
            );
        }
    }

    impl<'a, Message, Theme, Renderer> From<EmulatorWrapper> for Element<'a, Message, Theme, Renderer>
    where
        Renderer: renderer::Renderer,
    {
        fn from(emulator: EmulatorWrapper) -> Self {
            Self::new(emulator)
        }
    }
}




use iced::time::{self, milliseconds};
use iced::widget::{center, column, slider, text};
use iced::{Center, Element, Subscription};



//for handling keyboard input
use chip8emulator::emulator;
use iced::keyboard;
use iced_native::subscription;
use iced_native::Event;
use chippie_emulator::cpu::Cpu;
use chippie_emulator::rombuffer::RomBuffer;



pub fn main() -> iced::Result {
    iced::run(Example::update, Example::view)
}

struct Example {
    //this instantiates the widget
    emulator: Cpu,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    //this message should call the cycle method on the widget, maybe it has to be forwarded "inside" the widget somehow?
    //not sure how to connect it to external messages yet
    Tick,
}

impl Example {
    fn new() -> Self {
        //right now turning a string in to a rombuffer is fallible, solution: have the rom turn in to one that shows an error message
        //returning a valid rombuffer object regardless. Then we can get rid of the unwrap
        let rb: RomBuffer = "assets/2-ibm-logo.ch8".try_into().unwrap();
        let emulator = Cpu::new(&rb);
        //Returns an application with an emulator instance.
        Example { emulator }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                println!("yee refreshing the thing!");
            }
        }
    }

    // this ticks every .1 second as an example, should be used to cycle the widget
    fn subscription(&self) -> Subscription<Message> {
        time::every(milliseconds(100)).map(|_| Message::Tick)
    }
    
    fn view(&self) -> Element<'_, Message> {
        //Adds an emulator to a column that can draw itself
        let content = column![emulator("This should be the actual path to the rom file")]
            //decreases the maximum width of said emulator by 20px on all sides
            .padding(20)
            //clamps it to have a max width of 500
            .max_width(500)
            //center it
            .align_x(Center);
        center(content).into()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

