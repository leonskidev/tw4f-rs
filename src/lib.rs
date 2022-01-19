//! A tiny and safe abstraction over the [WASM-4] fantasy console.
//!
//! [WASM-4]: https://wasm4.org

#![no_std]
#![deny(missing_docs)]

/// Queries the current state of the gamepads.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Gamepad {
  /// The X button.
  X = 1 << 0,
  /// The Z button.
  Z = 1 << 1,

  /// The LEFT button.
  Left = 1 << 4,
  /// The RIGHT button.
  Right = 1 << 5,
  /// The UP button.
  Up = 1 << 6,
  /// The DOWN button.
  Down = 1 << 7,
}

impl Gamepad {
  const GAMEPADS: *const [u8; 4] = 0x16 as *const [u8; 4];

  /// Whether this button is currently being pressed.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory#gamepads)
  #[inline]
  pub fn pressed(self, player: Player) -> bool {
    unsafe { (*Self::GAMEPADS)[player as usize] & self as u8 == self as u8 }
  }
}

/// Useful for situations involving a specific player.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Player {
  /// Player 1.
  P1 = 0,
  /// Player 2.
  P2,
  /// Player 3.
  P3,
  /// Player 4.
  P4,
}

/// Queries the current state of the mouse.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Mouse {
  /// The LEFT button.
  Left = 1 << 0,
  /// The RIGHT button.
  Right = 1 << 1,
  /// The MIDDLE button.
  Middle = 1 << 2,
}

impl Mouse {
  const MOUSE_POSITION: *const [i16; 2] = 0x1a as *const [i16; 2];
  const MOUSE_BUTTONS: *const u8 = 0x1e as *const u8;

  /// Whether this button is currently being pressed.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#mouse_buttons)
  #[inline]
  pub fn pressed(self) -> bool {
    unsafe { *Self::MOUSE_BUTTONS & self as u8 == self as u8 }
  }

  /// The current X position.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#mouse_x)
  #[inline]
  pub fn x() -> i16 {
    unsafe { (*Self::MOUSE_POSITION)[0] }
  }

  /// The current Y position.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#mouse_y)
  #[inline]
  pub fn y() -> i16 {
    unsafe { (*Self::MOUSE_POSITION)[1] }
  }
}

/// Queries the current state of the palette.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Palette {
  /// Colour 1.
  C1 = 0,
  /// Colour 2.
  C2,
  /// Colour 3.
  C3,
  /// Colour 4.
  C4,
}

impl Palette {
  const PALETTE: *mut [u32; 4] = 0x04 as *mut [u32; 4];

  /// Returns the colour from the palette.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#palette)
  #[inline]
  pub fn load(self) -> Color {
    unsafe { Color::from_u32((*Self::PALETTE)[self as usize]) }
  }

  /// Sets the colour in the palette.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#palette)
  #[inline]
  pub fn store(self, color: Color) {
    unsafe { (*Self::PALETTE)[self as usize] = color.as_u32() }
  }
}

/// Represents a 24-bit colour.
#[derive(Clone, Copy)]
pub struct Color {
  /// The redness.
  pub r: u8,
  /// The greenness.
  pub g: u8,
  /// The blueness.
  pub b: u8,
}

impl Color {
  /// Create a new colour using RGB.
  #[inline]
  pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
    Self { r, g, b }
  }

  /// Converts this colour to a `u32`.
  pub const fn to_u32(self) -> u32 {
    ((self.r as u32) << 16) | ((self.g as u32) << 8) as u32 | self.b as u32
  }

  /// Converts a `u32` to a colour.
  pub const fn from_u32(v: u32) -> Self {
    Self {
      r: (v >> 16) as u8,
      g: (v >> 8) as u8,
      b: v as u8,
    }
  }
}

/// Queries the current state of the draw colours.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DrawColor {
  /// Colour 1.
  C1 = 0,
  /// Colour 2.
  C2,
  /// Colour 3.
  C3,
  /// Colour 4.
  C4,
}

impl DrawColor {
  const DRAW_COLORS: *mut u16 = 0x14 as *mut u16;

  /// Returns the palette index for the draw colour, or [`None`] for
  /// transparent.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#draw_colors)
  pub fn load(self) -> Option<Palette> {
    let offset = 4 * self as u8;
    let mask: u16 = 0b1111 << offset;

    match unsafe { *Self::DRAW_COLORS & mask } >> offset {
      0b0000 => None,
      0b0001 => Some(Palette::C1),
      0b0010 => Some(Palette::C2),
      0b0011 => Some(Palette::C3),
      0b0100 => Some(Palette::C4),
      _ => unreachable!(),
    }
  }

  /// Sets the palette index for the draw colour, or [`None`] for transparent.
  ///
  /// [WASM-4 Docs](https://wasm4.org/docs/reference/memory/#draw_colors)
  pub fn store(self, palette: Option<Palette>) {
    let offset = 4 * self as u8;
    let mask: u16 = 0b1111 << offset;
    let index: u16 = match palette {
      Some(color) => ((color as u8) + 1) as u16,
      None => 0,
    };

    unsafe {
      *Self::DRAW_COLORS ^= *Self::DRAW_COLORS & mask;
      *Self::DRAW_COLORS |= index << offset;
    }
  }
}

pub mod w4 {
  //! The built-in [WASM-4 functions].
  //!
  //! [WASM-4 functions]: https://wasm4.org/docs/reference/functions

  extern "C" {
    /// Copies pixels in memory into the framebuffer.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#blit-spriteptr-x-y-width-height-flags)
    pub fn blit(
      sprite: *const u8,
      x: i32,
      y: i32,
      width: i32,
      height: i32,
      flags: i32,
    );
    /// Copies pixels within a subsection of memory into the framebuffer.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#blitsub-spriteptr-x-y-width-height-srcx-srcy-stride-flags)
    #[link_name = "blitSub"]
    pub fn blit_sub(
      sprite: *const u8,
      x: i32,
      y: i32,
      width: i32,
      height: i32,
      src_x: i32,
      src_y: i32,
      stride: i32,
      flags: i32,
    );
    /// Draws a line between two points.
    ///
    /// Uses `DrawColor::C1` for the line.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#line-x1-y1-x2-y2)
    pub fn line(x1: i32, y1: i32, x2: i32, y2: i32);
    /// Draws a horizontal line.
    ///
    /// Uses `DrawColor::C1` for the line.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#hlinex-y-len)
    pub fn hline(x: i32, y: i32, len: i32);
    /// Draws a vertical line.
    ///
    /// Uses `DrawColor::C1` for the line.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#vlinex-y-len)
    pub fn vline(x: i32, y: i32, len: i32);
    /// Draws an oval.
    ///
    /// Uses `DrawColor::C1` for the fill and `DrawColor::C2` for the outline.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#oval-x-y-width-height)
    pub fn oval(x: i32, y: i32, width: i32, height: i32);
    /// Draws a rectangle.
    ///
    /// Uses `DrawColor::C1` for the fill and `DrawColor::C2` for the outline.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#rect-x-y-width-height)
    pub fn rect(x: i32, y: i32, width: i32, height: i32);
    /// Draws text using the built-in system font.
    ///
    /// Uses `DrawColor::C1` for the text and `DrawColor::C2` for the background.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#text-str-x-y)
    #[link_name = "textUtf8"]
    pub fn text(string: *const u8, len: i32, x: i32, y: i32);

    /// Plays a sound.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#sound)
    pub fn tone(frequency: i32, duration: i32, volume: i32, flags: i32);

    /// Read bytes from storage.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#diskr-destptr-size)
    pub fn diskr(dest: *const u8, size: i32) -> i32;
    /// Writes bytes to storage.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#diskw-srcptr-size)
    pub fn diskw(src: *const u8, size: i32) -> i32;

    /// Writes a message to the debug console.
    ///
    /// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#trace-str)
    #[link_name = "traceUtf8"]
    pub fn trace(text: *const u8, len: usize);
  }
}

/// Debug prints text to the terminal.
///
/// [WASM-4 Docs](https://wasm4.org/docs/reference/functions#trace-str)
#[inline]
pub fn trace<T: AsRef<str>>(text: T) {
  let text = text.as_ref();
  unsafe { w4::trace(text.as_ptr(), text.len()) }
}
