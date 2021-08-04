use super::SmphrParams;
use std::f64::consts::{PI, FRAC_PI_4};

static RIGHT_FOOT:     Point = Point { x: 10, y:  40 };
static LEFT_FOOT:      Point = Point { x: -10,  y: 40 };
static RIGHT_SHOULDER: Point = Point { x: -5, y: -10 };
static LEFT_SHOULDER:  Point = Point { x: 5, y: -10 };

static TRUC:           Point = Point { x: 0,   y: 20 };
static NECK:           Point = Point { x: 0, y: -10 };
static NOSE:           Point = Point { x: 0, y: -20 };
static HEAD_SIZE:      usize = (NECK.y - NOSE.y) as usize;
static ARM_LENGTH:     i32 = 30;
static FLAG_LENGTH:    i32 = 10;
static X_MARGIN:       i32 = 1;
static Y_MARGIN:       i32 = 1;

static HEAD_THICKNESS:     usize = 2;
static BODY_THICKNESS:     usize = 10;
static LEG_THICKNESS:      usize = 3;

static STICK_WIDTH:    i32 = 2*(LEFT_SHOULDER.x + ARM_LENGTH) + X_MARGIN;
static STICK_HEIGHT:   i32 = RIGHT_FOOT.y - RIGHT_SHOULDER.y
                             + ARM_LENGTH + Y_MARGIN;

// static WHITE: u8 = 0;
static BLACK: u8 = 1;
static RED:   u8 = 2;

//                                  a  b  c  d  e  f  g  h  i
//                                  j  k  l  m  n  o  p  q  r
//                                  s  t  u  v  w  x  y  z
/// Positions of the right arm for each letter
static RIGHT_ARM_POS: &[u8; 26] = &[1, 2, 3, 4, 0, 0, 0, 1, 1,
                                    4, 1, 1, 1, 1, 2, 2, 2, 2,
                                    2, 3, 3, 4, 5, 5, 3, 6];
/// Positions of the left arm for each letter
static LEFT_ARM_POS:  &[u8; 26] = &[0, 0, 0, 0, 5, 6, 7, 2, 3,
                                    6, 4, 5, 6, 7, 3, 4, 5, 6,
                                    7, 4, 5, 7, 6, 7, 6, 7];

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn from(x: i32, y: i32) -> Point {
        Point {
            x, 
            y,
        }
    }

    fn increment(&self, params: &SmphrParams) -> Result<Point, StickmanError> {
        if self.x + STICK_WIDTH >= params.width as i32 {
            // we could cut their legs
            // stickmen are not alive so that we can hurt them
            if self.y + STICK_HEIGHT > params.height as i32 {
                return Err(StickmanError::VerticalOverflow);
            }
            Ok(Point::from(STICK_WIDTH / 2, self.y + STICK_HEIGHT))
        } else {
            Ok(Point::from(self.x + STICK_WIDTH, self.y))
        }
    }
}

#[derive(Debug)]
pub enum StickmanError {
    InvalidCharacter(char),
    VerticalOverflow,
}

impl std::fmt::Display for StickmanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCharacter(c) => { 
                write!(f, "invalid character '{}'", c) }
            Self::VerticalOverflow => { write!(f, "vertical overflow") }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum StickmanType {
    Character,
    Space, 
    Unknown,
    CarriageReturn
}

pub struct Stickman {
    right_hand: u8,
    left_hand:  u8,
    cg:         Point,
    stype:      StickmanType,
}

impl Stickman {
    fn new() -> Stickman {
        Stickman {
            right_hand: 0,
            left_hand:  0,
            cg:         Point::from(0, 0),
            stype:      StickmanType::Unknown,
        }
    }

    // TODO: handle numeric digits
    pub fn first_from_letter(c: char) -> Result<Stickman, StickmanError> {
        if !c.is_ascii_alphanumeric() && c != ' ' && c != '\n' {
            return Err(StickmanError::InvalidCharacter(c));
        }

        let v = c.to_ascii_lowercase();
        let i = if v.is_alphabetic() { (v as u8 - 'a' as u8) as usize } 
                else { 0 };

        let mut s: Stickman = Stickman::new();

        s.stype = match c {
            ' ' => StickmanType::Space,
            '\n' => StickmanType::CarriageReturn,
            _ => StickmanType::Character,
        };

        match s.stype {
            StickmanType::Character => {},
            _ => { return Ok(s); }
        }

        s.right_hand = RIGHT_ARM_POS[i];
        s.left_hand = LEFT_ARM_POS[i];
        s.cg = Point::from(STICK_WIDTH / 2, STICK_HEIGHT / 2);
        s.stype = StickmanType::Character;

        Ok(s)
    }

    pub fn from_letter_with_prev(c: char, prev_pos: &Point, 
        params: &SmphrParams)
        -> Result<Stickman, StickmanError> {

        match Stickman::first_from_letter(c) {
            Err(e) => Err(e),
            Ok(s) => Ok(s.set_pos(prev_pos.increment(params)?))
        }
    }

    fn set_pos(&self, pos: Point) -> Stickman {
        Stickman {
            right_hand: self.right_hand,
            left_hand: self.left_hand,
            cg: pos,
            stype: self.stype,
        }
    }

    pub fn get_pos(&self) -> &Point {
        &self.cg
    }

    pub fn draw(&self, tab: &mut [u8], params: &SmphrParams) {
        match self.stype {
            StickmanType::Space => { return; },
            StickmanType::CarriageReturn => { return; },
            StickmanType::Unknown => { println!("?"); return; }
            _ => {}
        }
        self.draw_body(tab, params);

        let (tabw, tabh) = (params.width as usize, params.height as usize);
        self.draw_arm(self.right_hand, true, tab, tabw, tabh);
        self.draw_arm(self.left_hand, false, tab, tabw, tabh);
    }

    fn draw_body(&self, tab: &mut [u8], params: &SmphrParams) {
        let (x, y) = (self.cg.x, self.cg.y);
        let (tabw, tabh) = (params.width as usize, params.height as usize);
        // body:
        draw_line(x + NECK.x, y + NECK.y, x + TRUC.x, y + TRUC.y,
                  BODY_THICKNESS, BLACK, tab, tabw, tabh);

        draw_line(x + TRUC.x, y + TRUC.y,
                  x + LEFT_FOOT.x, y + LEFT_FOOT.y,
                  LEG_THICKNESS, BLACK, tab, tabw, tabh);
        draw_line(x + TRUC.x, y + TRUC.y,
                  x + RIGHT_FOOT.x, y + RIGHT_FOOT.y,
                  LEG_THICKNESS, BLACK, tab, tabw, tabh);
        // head
        draw_circle((x + NOSE.x) as usize, (y + NOSE.y) as usize, HEAD_SIZE, 
                    HEAD_THICKNESS, BLACK, tab, tabw, tabh);
    }

    fn draw_arm(&self, n: u8, is_right: bool, 
        tab: &mut [u8], tabw: usize, tabh: usize) {

        if n == 0 { return; }
        let (x, y) = (self.cg.x, self.cg.y);

        let alpha = FRAC_PI_4 * (n as f64 - 2.0);
        let (sina, cosa) = alpha.sin_cos();
        let (armsin, armcos) = (ARM_LENGTH as f64 * sina, 
                                ARM_LENGTH as f64 * cosa);
        let (fsin, fcos) = (FLAG_LENGTH as f64 * sina, 
                            FLAG_LENGTH as f64 * cosa);
        let shoulder = if is_right { RIGHT_SHOULDER } else { LEFT_SHOULDER };

        let (x0, y0) = (x + shoulder.x - armcos as i32, 
                        y + shoulder.y - armsin as i32);
        let (x1, y1) = (x0 + fcos as i32, 
                        y0 + fsin as i32);
        let (x2, y2) = if n <= 4 { (x1 - fsin as i32, 
                                    y1 + fcos as i32) }
                       else      { (x1 + fsin as i32,
                                    y1 - fcos as i32) };
        let (x3, y3) = if n <= 4 { (x0 - fsin as i32,
                                    y0 + fcos as i32) }
                       else      { (x0 + fsin as i32,
                                    y0 - fcos as i32) };
        
        // draw arm
        draw_line2((x + shoulder.x) as usize, (y + shoulder.y) as usize, 
                    x0 as usize, y0 as usize, BLACK, tab, tabw, tabh);        
        // draw flag lines
        if n <= 4 {
            draw_line2(x1 as usize, y1 as usize, x3 as usize, y3 as usize, 
                       BLACK, tab, tabw, tabh);
        } else {
            draw_line2(x0 as usize, y0 as usize, x3 as usize, y3 as usize, 
                       BLACK, tab, tabw, tabh);
        }
        draw_line2(x3 as usize, y3 as usize, x2 as usize, y2 as usize, BLACK, tab, tabw, tabh);
        draw_line2(x2 as usize, y2 as usize, x1 as usize, y1 as usize, BLACK, tab, tabw, tabh);
        // red triangle:
        if n <= 4 {
            fill_triangle(x0 as usize, y0 as usize, x1 as usize, y1 as usize,
                          x3 as usize, y3 as usize, RED, tab, tabw, tabh)
        } else {
            fill_triangle(x0 as usize, y0 as usize, x1 as usize, y1 as usize,
                          x2 as usize, y2 as usize, RED, tab, tabw, tabh)
        }
    }
}

/* Returns (x, y) such as they are bounded by tabw and tabh
 */
fn bound(x: usize, y: usize, tabw: usize, tabh: usize) 
    -> (usize, usize) {
    let x2 = if x >= tabw {  tabw-1  } else {  x  };
    let y2 = if y >= tabh {  tabh-1  } else {  y  };

    (x2, y2)
}

/** Draws the line (xi, yi) -- (xf, yf) in array tab. */
fn draw_line2(xi: usize, yi: usize, xf: usize, yf: usize, color: u8,
             tab: &mut [u8], tabw: usize, tabh: usize) {
    
    let (xi2, yi2) = bound(xi, yi, tabw, tabh);
    let (xf2, yf2) = bound(xf, yf, tabw, tabh);

    assert!(xi2 < tabw && xf2 < tabw);
    assert!(yi2 < tabh && yf2 < tabh);
    let (x1, x2, x_inversed) = if xf2 < xi2 { (xf2, xi2, true)   } 
                               else         { (xi2, xf2, false)  };
    let (y1, y2, y_inversed) = if yf2 < yi2 { (yf2, yi2, true)   }
                               else         { (yi2, yf2, false)  };
    
    if x1 == x2 {
        if y1 == y2 {   tab[y1*tabw + x1] = color; 
                        return }
        for i in y1..=y2 {   tab[i*tabw + x1] = color;   }
        return
    }
    if y1 == y2 {
        for i in x1..=x2 {   tab[y1*tabw + i] = color;   }
        return

    }

    let mut y = y1;
    let (dx, dy) = ((x2 as i32 - x1 as i32), (y2 as i32 - y1 as i32));
    let mut e: i32 = -dx;
    let (ex, ey) = (2*dy, -2*dx);


    if !x_inversed {
        if !y_inversed { // quadrant 4
            for x in x1..x2 {
                tab[y*tabw + x] = color;

                e += ex;
                while e >= 0 {
                    y += 1;
                    e += ey;
                    if e >= 0 {   tab[y*tabw + x] = color;   }
                }
            }
        }
        else { // quadrant 1 
            for x in x1..x2 {
                tab[(y1+y2-y)*tabw + x] = color;

                e += ex;
                while e >= 0 {
                    y += 1;
                    e += ey;
                    if e >= 0 {   tab[(y1+y2-y)*tabw + x] = color;   }
                }
            }
        }
    }
    else { 
        if !y_inversed { // quadrant 3
            for x in x1..x2 {
                tab[y*tabw + x1+x2-x] = color;

                e += ex;
                while e >= 0 {
                    y += 1;
                    e += ey;
                    if e >= 0 {   tab[y*tabw + x1+x2-x] = color;   }
                }
            }
        }
        else { // quadrant 2
            for x in x1..x2 {
                tab[(y1+y2-y)*tabw + x1+x2-x] = color;

                e += ex;
                while e >= 0 {
                    y += 1;
                    e += ey;
                    if e >= 0 {   tab[(y1+y2-y)*tabw + x1+x2-x] = color;   }
                }
            }
        }
    }
}

fn draw_triangle(xa: usize, ya: usize,
                 xb: usize, yb: usize,
                 xc: usize, yc: usize, color: u8, tab: &mut [u8],
                 tabw: usize, tabh: usize) {

    draw_line2(xa, ya, xb, yb, color, tab, tabw, tabh);
    draw_line2(xa, ya, xc, yc, color, tab, tabw, tabh);
    draw_line2(xb, yb, xc, yc, color, tab, tabw, tabh);
}

fn fill_triangle(xa: usize,   ya: usize,
                 xb: usize,   yb: usize, 
                 xc: usize,   yc: usize, color: u8, tab: &mut [u8], 
                 tabw: usize, tabh: usize) {

    draw_triangle(xa, ya, xb, yb, xc, yc, color, tab, tabw, tabh);

    let (x0, y0) = ((xa + xb + xc) / 3, (ya + yb + yc) / 3);
    let mut queue = Vec::<(usize, usize)>::new();
    queue.push((x0, y0));
    if tab[y0 * tabw + x0] == color { return; }
    while let Some(p) = queue.pop() {
        check(bound(p.0+1, p.1, tabw, tabh), color, tab, tabw, &mut queue);
        if p.0 > 0 {
            check(bound(p.0-1, p.1, tabw, tabh), color, tab, tabw, &mut queue);
        }
        check(bound(p.0, p.1+1, tabw, tabh), color, tab, tabw, &mut queue);
        if p.1 > 0 {
            check(bound(p.0, p.1-1, tabw, tabh), color, tab, tabw, &mut queue);
        }
    }
}

fn check((x, y): (usize, usize), color: u8, tab: &mut [u8], tabw: usize, 
         queue: &mut Vec<(usize, usize)>) {

    if tab[y * tabw + x] != color {
        queue.push((x, y));
    }

    tab[y * tabw + x] = color;
}

fn draw_line(xi: i32, yi: i32, xf: i32, yf: i32, t: usize, color: u8,
             tab: &mut [u8], tabw: usize, tabh: usize) {

    let u = t / 2;
    let alpha = ((yf - yi) as f64).atan2((xf - xi) as f64);
    let (sina, cosa) = alpha.sin_cos();

    let (x0, y0) = ((xi as f64 + u as f64 * sina) as usize,
                    (yi as f64 + u as f64 * cosa) as usize);
    let (x1, y1) = ((xi as f64 - u as f64 * sina) as usize,
                    (yi as f64 - u as f64 * cosa) as usize);
    let (x2, y2) = ((xf as f64 + u as f64 * sina) as usize,
                    (yf as f64 + u as f64 * cosa) as usize);
    let (x3, y3) = ((xf as f64 - u as f64 * sina) as usize,
                    (yf as f64 - u as f64 * cosa) as usize);

    if t > 2 {
        fill_triangle(x0, y0, x1, y1, x2, y2, color, tab, tabw, tabh);
        fill_triangle(x2, y2, x3, y3, x1, y1, color, tab, tabw, tabh)
    } else {
        draw_triangle(x0, y0, x1, y1, x2, y2, color, tab, tabw, tabh);
        draw_triangle(x2, y2, x3, y3, x1, y1, color, tab, tabw, tabh)
    }
}

fn draw_circle(xc: usize, yc: usize, r: usize, t: usize, color: u8,
               tab: &mut [u8], tabw: usize, tabh: usize) {

    for i in 0..t {
        draw_circle2(xc, yc, r + i, color, tab, tabw, tabh);
    }
}
fn draw_circle2(xc: usize, yc: usize, r: usize, color: u8, 
               tab: &mut [u8], tabw: usize, tabh: usize) {
    /* we can expect about 2 * pi * r pixels to be 'on' for a circle of
       radius r, we'll upper approximate 2 * pi by 20 */
    
    let rf64 = r as f64;
    let n = 20 * r;
    for i in 0..n {
        let theta = 2.0 * PI * i as f64/ n as f64;
        let dx = (theta.cos() * rf64).round() as i32; 
        let dy = (theta.sin() * rf64).round() as i32;
        let (x, y) = bound((xc as i32 + dx) as usize, 
                           (yc as i32 + dy) as usize, tabw, tabh);
        tab[y*tabw + x] = color;
    }
}

mod test {
    #[test]
    fn truc() {
        let x = (2.0 as f64).atan2(0.0);
        println!("x: {}", x); // output: pi/2
    }
}