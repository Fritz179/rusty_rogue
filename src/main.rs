use std::borrow::BorrowMut;
use std::cell::RefCell;

use tekenen::{Tekenen, colors};
use tekenen::platform::{Platform, PlatformTrait, Event, IntervalDecision, Keycode};

use rand::Rng;
const MAP_SIZE: usize = 16;
const TILE_SIZE: i32 = 32;

#[derive(Debug)]
enum Items {
    Food,
    Bandage,
}

use once_cell::sync::Lazy;

static CONCRETE: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/concrete.png").unwrap() });
static WALL: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/wall.png").unwrap() });
static DOOR_CLOSED: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/door_closed.png").unwrap() });
static DOOR_OPEN: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/door_open.png").unwrap() });
static PLAYER_SPRITE: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/player.png").unwrap() });
static FOOD: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/food.png").unwrap() });
static BANDAGE: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/bandage.png").unwrap() });
static FLOOR: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/floor.png").unwrap() });
static SLOT: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/slot.png").unwrap() });
static SELECTED_SLOT: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/selected_slot.png").unwrap() });
static SOURCE_SLOT: Lazy<Tekenen> = Lazy::new(|| { Platform::load_image("./src/assets/source_slot.png").unwrap() });

impl Items {
    fn get_sprite(&self) -> &Tekenen {
        match self {
            Items::Bandage => {
                &BANDAGE
            },
            Items::Food => {
                &FOOD
            }
        }
    }
}

#[derive(Debug)]
struct ItemContainer<const SIZE: usize> {
    items: RefCell<[Option<Items>; SIZE]>
}

impl<const SIZE: usize> ItemContainer<SIZE> {
    fn new() -> Self {
        Self {
            items: RefCell::new(std::array::from_fn(|_| None))
        }
    }

    fn add(&self, item: Items) {
        let mut container = self.items.borrow_mut();
        container[0] = Some(item);
    }
}

#[derive(Debug)]
struct FloorTile(Box<ItemContainer9>);

#[derive(Debug)]
struct RoadTile(Box<ItemContainer9>);

impl FloorTile {
    fn new() -> Self{
        Self(Box::new(ItemContainer::new()))
    }
}

impl RoadTile {
    fn new() -> Self{
        Self(Box::new(ItemContainer::new()))
    }
}


#[derive(Debug)]
enum Tiles {
    Road(RoadTile),
    Wall,
    Floor(FloorTile),
    Door(bool),
}

impl Tiles {
    fn get_sprite(&self) -> &Tekenen {
        match self {
            Self::Wall => {
                &WALL
            },
            Self::Door(open) => {
                if *open {
                    &DOOR_OPEN
                } else {
                    &DOOR_CLOSED
                }
            },
            Self::Road(_) => {
                &CONCRETE
            },
            Self::Floor(_) => {
                &FLOOR
            },
        }
    }

    fn get_items(&self) -> Option<&ItemContainer9> {
        match self {
            Self::Road(RoadTile(items)) => Some(items),
            Self::Wall => None,
            Self::Door(_) => None,
            Self::Floor(FloorTile(items)) => Some(items)
        }
    }

    fn get_items_mut(&mut self) -> Option<&mut ItemContainer9> {
        match self {
            Self::Road(RoadTile(items)) => Some(items),
            Self::Wall => None,
            Self::Door(_) => None,
            Self::Floor(FloorTile(items)) => Some(items)
        }
    }
}

type ItemContainer9 = ItemContainer<9>;

struct MapHandler {
    map: Vec<Tiles>,
    width: usize,
    height: usize,
}

impl MapHandler {
    fn new(width: usize, height: usize) -> Self {
        let mut map = Vec::with_capacity(width * height);

        for _ in 0..width*height {
            map.push(Tiles::Road(RoadTile::new()))
        }

        Self {
            map,
            width,
            height
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn is_in(&self, x: usize, y: usize) -> bool {
        // comparison is useless due to type limits of usize
        // x >= 0 &&
        // y >= 0 &&
        x < self.width() &&
        y < self.height()
    }

    fn get_tile_at<T>(&self, x: T, y: T) -> Option<&Tiles> where T: TryInto<usize> {
        let x: usize = x.try_into().ok()?;
        let y: usize = y.try_into().ok()?;

        if !self.is_in(x, y) {
            None
        } else {
            let i = y * self.width() + x;
            Some(&self.map[i])
        }
    }

    fn get_tile_at_mut<T>(&mut self, x: T, y: T) -> Option<&mut Tiles> where T: TryInto<usize> {
        let x: usize = x.try_into().ok()?;
        let y: usize = y.try_into().ok()?;

        if !self.is_in(x, y) {
            None
        } else {
            let i = y * self.width() + x;
            Some(&mut self.map[i])
        }
    }

    fn set_tile_at(&mut self, x: usize, y: usize, tile: Tiles) {
        if !self.is_in(x, y) {
            panic!("Map acces out of bounds!")
        }

        let i = y * self.width() + x;
        self.map[i] = tile;
    }
}

fn generate_map() -> MapHandler {
    let mut rng = rand::thread_rng();

    let mut map = MapHandler::new(MAP_SIZE, MAP_SIZE);

    for x in 5..=MAP_SIZE-5 {
        for y in 5..=MAP_SIZE-5 {

            let tile = FloorTile::new();

            if rng.gen_bool(0.1) { tile.0.add(Items::Bandage) }
            if rng.gen_bool(0.1) { tile.0.add(Items::Food) }

            map.set_tile_at(x, y, Tiles::Floor(tile))
        }
    }

    for x in 4..MAP_SIZE-4 {
        map.set_tile_at(x, 4, Tiles::Wall);
        map.set_tile_at(x, MAP_SIZE - 4, Tiles::Wall);
    }

    for y in 4..MAP_SIZE-4 {
        map.set_tile_at(4, y, Tiles::Wall);
        map.set_tile_at(MAP_SIZE - 4, y, Tiles::Wall);
    }

    let container = ItemContainer9::new();
    container.add(Items::Food);

    map.set_tile_at(0, 0, Tiles::Road(RoadTile(Box::new(container))));

    map.set_tile_at(8, 4, Tiles::Door(false));

    map
}

enum CompundAction {
    SwapItem(i32)
}

struct Stat {
    max: i32,
    value: i32,
}

impl std::cmp::PartialOrd<i32> for Stat {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

impl std::cmp::PartialEq<i32> for Stat {
    fn eq(&self, other: &i32) -> bool {
        self.value.eq(other)
    }
}

impl std::ops::AddAssign<i32> for Stat {
    fn add_assign(&mut self, rhs: i32) {
        self.value += rhs;

        if self.value > self.max {
            self.value = self.max
        }
    }
}

impl std::ops::SubAssign<i32> for Stat {
    fn sub_assign(&mut self, rhs: i32) {
        if self.value < rhs {
            self.value = 0
        } else {
            self.value -= rhs
        }
    }
}

impl Stat {
    fn new(value: i32) -> Self {
        Self {
            value,
            max: value
        }
    }

    fn is_maxed(&self) -> bool {
        self.value == self.max
    }
}

struct Player {
    x: i32,
    y: i32,
    items: ItemContainer9,
    selected_slot: i32,
    action: Option<CompundAction>,
    health: Stat,
    hunger: Stat,
    stamina: Stat
}

fn main() {
    assert_eq!(std::mem::size_of::<Tiles>(), 16);

    let mut window = Platform::new(800, 600).unwrap();
    let mut tek = Tekenen::new(800, 600);

    let mut map = generate_map();
    let mut player = Player {
        x: 2,
        y: 2,
        items: ItemContainer9::new(),
        selected_slot: 0,
        action: None,
        health: Stat::new(10),
        hunger: Stat::new(10),
        stamina: Stat::new(10),
    };

    let mut turns = vec!["Player is born!".to_owned()];

    Platform::set_interval(move || {
        let mut update_turn = |action: String| {
            turns.push(action);
        };

        while let Some(event) = window.read_events() {
            match event {
                Event::Quit => {
                    return IntervalDecision::Stop
                },
                Event::KeyDown { char: Some(char), .. } => {
                    let bound = |val: i32, min: i32, max: i32| -> i32 {
                        if val > min {
                            if val < max {
                                val
                            } else {
                                max
                            }
                        } else {
                            min
                        }
                    };

                    let mut move_player = |xv: i32, yv: i32| {
                        let x = bound(player.x + xv, 0, MAP_SIZE as i32 - 1);
                        let y = bound(player.y + yv, 0, MAP_SIZE as i32 - 1);

                        if player.stamina == 0 {
                            return
                        }

                        let tile = map.get_tile_at(x, y);

                        if let Some(Tiles::Wall) = tile {
                            return
                        }

                        if let Some(Tiles::Door(false)) = tile {
                            return
                        }

                        player.x = x;
                        player.y = y;

                        // player.stamina -= 1;
                        player.hunger -= 1;

                        update_turn(format!("Player moved to ({x}/{y})"));
                    };
                    
                    match char {
                        'w' => move_player(0, -1),
                        'a' => move_player(-1, 0),
                        's' => move_player(0, 1),
                        'd' => move_player(1, 0),
                        'p' => Platform::save_image("./screenshot.png", &tek).unwrap(),
                        'm' => {
                            player.action = Some(CompundAction::SwapItem(player.selected_slot));
                            let ground_items = map.get_tile_at(player.x, player.y).unwrap().get_items();

                            let (begin, container) = if player.selected_slot >= 9 {
                                (0, player.items.items.borrow())
                            } else if let Some(items) = ground_items {
                                (9, items.items.borrow())
                            } else {
                                panic!("Not standing on tile with items!, {:?}", map.get_tile_at_mut(player.x, player.y).unwrap());
                            };

                            let mut found = false;
                            for i in 0..9 {
                                if container[i].is_none() {
                                    player.selected_slot = begin + i as i32;
                                    found = true;
                                    break
                                }
                            }

                            if !found {
                                player.selected_slot = if player.selected_slot == begin { begin + 1 } else { begin }
                            }
                        },
                        'e' => {
                            for xd in -1..=1 {
                                for yd in -1..=1 {
                                    if xd == 0 && yd == 0 {
                                        continue;
                                    }

                                    let xi = player.x + xd;
                                    let yi = player.y + yd;
                                    let target = map.get_tile_at_mut(xi, yi);

                                    if let Some(Tiles::Door(state)) = target {
                                        *state = !*state;

                                        let action = if *state {
                                            "opened"
                                        } else {
                                            "closed"
                                        };

                                        update_turn(format!("Player {action} the door."));
                                    }
                                }
                            }
                        },
                        'u' => {
                            let ground_items = map.get_tile_at(player.x, player.y).unwrap().get_items();

                            let (mut item, i) = if player.selected_slot < 9 {
                                (player.items.items.borrow_mut(), player.selected_slot)
                            } else if let Some(items) = ground_items {
                                (items.items.borrow_mut(), player.selected_slot - 9)
                            } else {
                                panic!("Not standing on tile with items!, {:?}", map.get_tile_at_mut(player.x, player.y).unwrap());
                            };

                            let item = &mut item[i as usize];

                            match item {
                                Some(Items::Bandage) => {
                                    if !player.health.is_maxed() {
                                        player.health += 10;
                                        *item = None
                                    }
                                },
                                Some(Items::Food) => {
                                    if !player.hunger.is_maxed() {
                                        player.hunger += 10;
                                        *item = None
                                    }
                                },
                                None => { }
                            }
                        }
                        _ => continue
                    }
                },
                Event::KeyDown { keycode: Some(keycode), ..} => {
                    let slot = &mut player.selected_slot;

                    match keycode {
                        Keycode::ArrowUp => if *slot >= 9 { *slot -= 9 },
                        Keycode::ArrowDown => if *slot < 9 { *slot += 9 },
                        Keycode::ArrowLeft => *slot -= 1,
                        Keycode::ArrowRight => *slot += 1,
                        _ => { }
                    }

                    if *slot < 0 { *slot = 0 }
                    if *slot > 17 { *slot = 17 }

                    match keycode {
                        Keycode::Escape => {
                            player.action = None
                        }
                        Keycode::Enter => {
                            if let Some(action) = &player.action {
                                match action {
                                    &CompundAction::SwapItem(from) => {

                                        if from == player.selected_slot {
                                            break
                                        }

                                        let mut player_items = player.items.items.borrow_mut();
                                        let mut ground_items = map.get_tile_at(player.x, player.y).unwrap().get_items().unwrap().items.borrow_mut();

                                        let mut temp = None;

                                        let mut swap = |slot: i32| {
                                            if slot < 9 {
                                                std::mem::swap(&mut player_items[slot as usize], &mut temp)
                                            } else {
                                                std::mem::swap(&mut ground_items[slot as usize - 9], &mut temp)
                                            }
                                        };

                                        swap(from);
                                        swap(player.selected_slot);
                                        swap(from);

                                        update_turn(format!("Swapped item from: {from}, to: {}", player.selected_slot));
                                    }
                                }

                                player.action = None
                            }
                        },
                        _ => { }
                    }
                },
                _ => { }
            }
        }

        // Update player stats
        if player.hunger == 0 {
            player.health -= 1
        }

        tek.background(colors::GRAY);

        
        // Map
        let cx = player.x;
        let cy = player.y;

        for x in cx-5..cx+5 {
            if x < 0 || x >= MAP_SIZE as i32 {
                continue;
            }

            for y in cy-5..cy+5 {
                if y < 0 || y >= MAP_SIZE as i32 {
                    continue;
                }

                let tile = map.get_tile_at(x, y).unwrap();

                let x = x - cx + 5;
                let y = y - cy + 5;

                let texture = tile.get_sprite();
                tek.draw_image(x * TILE_SIZE, y * TILE_SIZE, texture);


                if let Some(items) = tile.get_items() {
                    let items = items.items.borrow();

                    for i in 0..9 {
                        if let Some(item) = &items[i] {
                            let texture = Items::get_sprite(item);
                            tek.draw_image(x * TILE_SIZE, y * TILE_SIZE, texture);
                            break
                        }
                    }
                }
            }
        }

        // Player
        tek.draw_image(5 * TILE_SIZE, 5 * TILE_SIZE, &PLAYER_SPRITE);

        // Turns recap
        let start = if turns.len() > 5 {
            turns.len() - 5
        } else {
            0
        };

        for i in start..turns.len() {
            let y = 580 - 20 * (turns.len() - i) ;
            tek.draw_text(&format!("{i}: {}", &turns[i]), 10, y as i32);
        }

        let get_slot_texture = |slot: i32| -> &Tekenen {
            if player.selected_slot == slot {
                &SELECTED_SLOT
            } else if let Some(CompundAction::SwapItem(source_slot)) = player.action {
                if source_slot == slot {
                    &SOURCE_SLOT
                } else {
                    &SLOT
                }
            } else {
                &SLOT
            }
        };

        // player info
        tek.draw_text("Player inventory:", 15 * TILE_SIZE, 6);
        for i in 0..9 {
            let texture = get_slot_texture(i);
            tek.draw_image((15 + i) * TILE_SIZE, 1 * TILE_SIZE, texture);

            let item = &player.items.items.borrow()[i as usize];

            if let Some(item) = item {
                let sprite = item.get_sprite();
                tek.draw_image((15 + i) * TILE_SIZE, 1 * TILE_SIZE, sprite);
            }
        }

        let ground_items = map.get_tile_at(player.x, player.y).unwrap().get_items();
        tek.draw_text("On ground:", 15 * TILE_SIZE, 2 * TILE_SIZE + 6);
        for i in 0..9 {
            let texture = get_slot_texture(i + 9);
            tek.draw_image((15 + i) * TILE_SIZE, 3 * TILE_SIZE, texture);

            if let Some(items) = ground_items {
                let item = &items.items.borrow()[i as usize];

                if let Some(item) = item {
                    let sprite = item.get_sprite();
                    tek.draw_image((15 + i) * TILE_SIZE, 3 * TILE_SIZE, sprite);
                }
            }
        }


        window.display_pixels(tek.get_pixels());

        IntervalDecision::Repeat
    }, 60)
}
