//! Retain expensive CSS rasterization and text metrics; canvas calls stay live.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub rebuilds: u64,
    pub reuses: u64,
    pub paint_commands_built: u64,
    pub text_measurements: u64,
}

enum Command {
    Fill {
        rect: Rect,
        color: Color,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        clip: Rect,
    },
    Image {
        src: String,
        source: Rect,
        destination: Rect,
        clip: Rect,
    },
    Canvas {
        id: Option<String>,
        bounds: Rect,
        content: Rect,
        clip: Rect,
    },
}

pub(super) struct CachedUi {
    pub key: (u64, u32),
    commands: Vec<Command>,
}
impl CachedUi {
    pub fn paint<R: Renderer>(&self, renderer: &mut R) -> Result<(), R::Error> {
        for command in &self.commands {
            match command {
                Command::Fill { rect, color } => renderer.draw(PaintCommand::FillRect {
                    rect: *rect,
                    color: *color,
                })?,
                Command::Text {
                    text,
                    x,
                    y,
                    size,
                    color,
                    clip,
                } => renderer.draw(PaintCommand::Text {
                    text,
                    x: *x,
                    y: *y,
                    size: *size,
                    color: *color,
                    clip: *clip,
                })?,
                Command::Image {
                    src,
                    source,
                    destination,
                    clip,
                } => renderer.draw_image(ImageDraw {
                    src,
                    source: *source,
                    destination: *destination,
                    clip: *clip,
                })?,
                Command::Canvas {
                    id,
                    bounds,
                    content,
                    clip,
                } => renderer.draw_canvas(CanvasRegion {
                    id: id.as_deref(),
                    bounds: *bounds,
                    content: *content,
                    clip: *clip,
                })?,
            }
        }
        Ok(())
    }
}

struct Recorder<'a, 'b> {
    backend: &'a mut RaylibBackend<'b>,
    scale: f32,
    commands: Vec<Command>,
}
impl Renderer for Recorder<'_, '_> {
    type Error = BackendError;
    fn raster_scale(&self) -> (f32, f32) {
        (self.scale, self.scale)
    }
    fn measure_text(&mut self, text: &str, size: f32) -> Result<(f32, f32), BackendError> {
        self.backend.cache_stats.text_measurements += 1;
        let text = self.backend.text(text, size * self.scale)?;
        Ok((text.advance / self.scale, text.height / self.scale))
    }
    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.backend
            .images
            .get(src)
            .map(|t| (t.width() as u32, t.height() as u32))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), BackendError> {
        self.commands.push(match command {
            PaintCommand::FillRect { rect, color } => Command::Fill { rect, color },
            PaintCommand::Text {
                text,
                x,
                y,
                size,
                color,
                clip,
            } => Command::Text {
                text: text.to_owned(),
                x,
                y,
                size,
                color,
                clip,
            },
        });
        Ok(())
    }
    fn draw_image(&mut self, image: ImageDraw<'_>) -> Result<(), BackendError> {
        self.commands.push(Command::Image {
            src: image.src.to_owned(),
            source: image.source,
            destination: image.destination,
            clip: image.clip,
        });
        Ok(())
    }
    fn draw_canvas(&mut self, region: CanvasRegion<'_>) -> Result<(), BackendError> {
        self.commands.push(Command::Canvas {
            id: region.id.map(str::to_owned),
            bounds: region.bounds,
            content: region.content,
            clip: region.clip,
        });
        Ok(())
    }
}

pub(super) fn record(
    backend: &mut RaylibBackend<'_>,
    ui: &Ui,
    scale: f32,
    key: (u64, u32),
) -> Result<CachedUi, BackendError> {
    let mut recorder = Recorder {
        backend,
        scale,
        commands: Vec::new(),
    };
    ui.render(&mut recorder)?;
    recorder.backend.cache_stats.rebuilds += 1;
    recorder.backend.cache_stats.paint_commands_built += recorder.commands.len() as u64;
    Ok(CachedUi {
        key,
        commands: recorder.commands,
    })
}
