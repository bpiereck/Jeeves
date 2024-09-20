/************** An Image Buffer **************
 * Represented as a 4-tree of client buffers *
 *********************************************/

/*
Constraints:
- each painter gets 40x40 pixels
- once in place on the canvas, the painter should not move unless painters around it are removed
- final canvas must be square
- maximum final canvas size is 8 x 8 (40 x 40 pixel) painters = 320 x 320 pixels (64 painters)
*/

pub const BUFFER_PIXELS: usize = 40;
const CLIENT_PIXELS: usize = 40 * 40;
const PIXEL_SIZE: usize = 4;

static GRID_POSITION: [(usize, usize); 64] = [
    // 1       2       3       4       5       6       7       8       9
    (0, 0), (1, 0), (0, 1), (1, 1), (2, 0), (2, 1), (0, 2), (1, 2), (2, 2),

    //10      11      12      13      14      15      16      17      18
    (3, 0), (3, 1), (3, 2), (0, 3), (1, 3), (2, 3), (3, 3), (4, 0), (4, 1),

    //19      20      21      22      23      24      25      26      27
    (4, 2), (4, 3), (0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (5, 0), (5, 1),

    //28      29      30      31      32      33      34      35      36
    (5, 2), (5, 3), (5, 4), (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5),

    //37      38      39      40      41      42      43      44      45
    (6, 0), (6, 1), (6, 2), (6, 3), (6, 4), (6, 5), (0, 6), (1, 6), (2, 6),

    //46      47      48      49      50      51      52      53      54
    (3, 6), (4, 6), (5, 6), (6, 6), (7, 0), (7, 1), (7, 2), (7, 3), (7, 4),

    //55      56      57      58      59      60      61      62      63
    (7, 5), (7, 6), (0, 7), (1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7),

    //64     
    (7, 7)
];

struct Client {
    id: u64,
    buffer: [u8; CLIENT_PIXELS * PIXEL_SIZE]
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id={}", self.id)
    }
}

pub struct Buffer {
    clients: Vec<Client>,
    pixels: Vec<u8>,
}

impl<'a> From<&'a Buffer> for &'a Vec<u8> {
    fn from(buffer: &'a Buffer) -> Self {
        &buffer.pixels
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        let pixels = Vec::with_capacity(64 * CLIENT_PIXELS * PIXEL_SIZE);
        Buffer {
            clients: Vec::new(),
            pixels,
        }
    }

    pub fn insert(&mut self, id: u64) {
        if self.clients.iter().any(|client| client.id == id) {
            return;
        }

        let pre_dim = self.dim();

        let client = Client {id, buffer: [0; CLIENT_PIXELS * PIXEL_SIZE]};
        self.clients.push(client);

        let post_dim = self.dim();
        if pre_dim < post_dim {
            // Need to re-size and re-render
            self.pixels.resize(post_dim * post_dim * PIXEL_SIZE, 0);
            self.full_render();
        }
    }

    pub fn remove(&mut self, id: u64) {
        if let Some(i) = self.clients.iter().position(|c| c.id == id) {
            self.clients.remove(i);
        }
    }

    pub fn update(&mut self, id: u64, data: Vec<u8>) {
        println!("Attempting to copy {} bytes to a buffer of {} bytes.", data.len(), CLIENT_PIXELS * PIXEL_SIZE);
        let client = self.clients.iter().position(|c| c.id == id); //.enumerate().find(|(_, c)| c.id == id);
        if let Some(i) = client {
            if data.len() > CLIENT_PIXELS * PIXEL_SIZE {
                eprintln!("Warning: data is larger than buffer")
            }
            self.clients[i].buffer.copy_from_slice(&data[0..(CLIENT_PIXELS * PIXEL_SIZE)]);
            if let Some((x, y)) = coordinate_of(i) {
                self.blit(x * BUFFER_PIXELS, y * BUFFER_PIXELS, self.clients[i].buffer);
            }
        } else {
            eprintln!("Error: could not find the client to update pixels.");
        }
    }

    pub fn dim(&self) -> usize {
        (self.clients.len() as f32).sqrt().ceil() as usize * BUFFER_PIXELS
    }

    fn blit(&mut self, x: usize, y: usize, source: [u8; CLIENT_PIXELS * PIXEL_SIZE]) {
        let copy_width = BUFFER_PIXELS * PIXEL_SIZE;
        let buffer_width = self.dim() * PIXEL_SIZE;
        let start = y * buffer_width + x * PIXEL_SIZE;
        (0..BUFFER_PIXELS).for_each(|y_off| {
            let dst_from = start + y_off * buffer_width;
            let dst_to = dst_from + copy_width;
            let src_from = y_off * copy_width;
            let src_to = src_from + copy_width;
            self.pixels[dst_from..dst_to].copy_from_slice(&source[src_from..src_to])
        })
    }

    fn full_render(&mut self) {
        // The trick here is to position the client buffers correctly
        let render_data: Vec<_> = self.clients
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if let Some((x, y)) = coordinate_of(i + 1) {
                    Some((x * BUFFER_PIXELS, y * BUFFER_PIXELS, c.buffer))
                } else {
                    None
                }
            })
            .collect();
        for (x, y, buffer) in render_data {
            self.blit(x, y, buffer);
        }
    }
}


fn coordinate_of(i: usize) -> Option<(usize, usize)> {
    if i > 0 && i <= 64 {
        Some(GRID_POSITION[i - 1])
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_position() {
        assert_eq!(None, coordinate_of(0));
        assert_eq!(Some((0, 0)), coordinate_of(1));
        assert_eq!(Some((6, 4)), coordinate_of(41));
        assert_eq!(None, coordinate_of(65));
    }

    #[test]
    fn test_update_pixels() {
        let mut buf = Buffer::new();
        buf.insert(0);
        buf.update(0, vec![255; CLIENT_PIXELS * PIXEL_SIZE]);
        assert_eq!(Vec::<u8>::from(&buf), vec![255; CLIENT_PIXELS * PIXEL_SIZE]);
    }

    #[test]
    fn test_four_clients() {
        let mut buf = Buffer::new();
        buf.insert(0);
        buf.insert(1);
        buf.insert(2);
        buf.insert(3);
        buf.update(0, vec![50; CLIENT_PIXELS * PIXEL_SIZE]);
        buf.update(1, vec![100; CLIENT_PIXELS * PIXEL_SIZE]);
        buf.update(2, vec![150; CLIENT_PIXELS * PIXEL_SIZE]);
        buf.update(3, vec![200; CLIENT_PIXELS * PIXEL_SIZE]);

        let mut one: Vec<u8> = vec![50; BUFFER_PIXELS * PIXEL_SIZE];
        one.extend(vec![100; BUFFER_PIXELS * PIXEL_SIZE]);
        let one_expected: Vec<u8> = one.iter().cycle().take(2 * CLIENT_PIXELS * PIXEL_SIZE).cloned().collect();
        let mut two: Vec<u8> = vec![150; BUFFER_PIXELS * PIXEL_SIZE];
        two.extend(vec![200; BUFFER_PIXELS * PIXEL_SIZE]);
        let mut two_expected: Vec<u8> = two.iter().cycle().take(2 * CLIENT_PIXELS * PIXEL_SIZE).cloned().collect();
        let mut expected = one_expected;
        expected.append(&mut two_expected);

        assert_eq!(expected.len(), 4 * CLIENT_PIXELS * PIXEL_SIZE);
        
        assert_eq!(Vec::<u8>::from(&buf), expected);
        //assert_eq!(Vec::<u8>::from(&buf)[4 * CLIENT_PIXELS * PIXEL_SIZE - 1], expected[4 * CLIENT_PIXELS * PIXEL_SIZE - 1])
    }
}
