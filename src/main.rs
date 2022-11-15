use rand::random;
use crossterm::{
    execute,
    cursor::{
        Hide,
        Show,
        MoveTo
    },
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
        SetTitle,
        size,
        Clear,
        ClearType
    },
    style::{
        SetBackgroundColor,
        SetForegroundColor,
        ResetColor,
        Color,
        Print
    },
    event::{
        read,
        poll,
        Event,
        KeyCode
    },
    Result
};
use std::{
    time::{
        Duration, 
        Instant
    },
    io::{
        stdout, 
        Stdout
    }
};

#[derive(Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn kind(&self) -> DirectionKind {
        match self {
            Direction::Up => DirectionKind::Vertical,
            Direction::Down => DirectionKind::Vertical,
            _ => DirectionKind::Horizontal
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum DirectionKind {
    Horizontal,
    Vertical
}

#[derive(Clone, PartialEq, Eq)]
struct SnakeGameCord {
    x: usize,
    y: usize
}

impl SnakeGameCord {
    fn move_direction(&mut self, direction: &Direction) {
        match direction {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1
        }
    }

    fn moved_direction(&self, direction: &Direction) -> SnakeGameCord {
        let mut new_cord = self.clone();

        new_cord.move_direction(direction);

        new_cord
    }
}

struct SnakeGame {
    data: Vec<Vec<i32>>,
    snake_head_pos: SnakeGameCord,
    snake_len: u32,
    snake_saturation_len: u32,
    grew_last_tick: bool,
    dead: bool,
    game_grow_rate: u32,
    max_apple_count: u32,
    min_apple_count: u32,
    ticks_between_apple_spawn: u32,
    ticks_since_last_apple_spawned: u32
}

impl SnakeGame {
    fn create(width: usize, height: usize, game_grow_rate: u32, max_apple_count: u32, min_apple_count: u32, ticks_between_apple_spawn: u32) -> SnakeGame {
        let row = vec![0].repeat(width);

        let mut data = vec![];

        for _ in 0..height {
            data.push(row.clone());
        }

        SnakeGame { 
            data, 
            snake_head_pos: SnakeGameCord { x: width/3, y: height/2 }, 
            snake_len: 0,
            snake_saturation_len: 3,
            grew_last_tick: true,
            dead: false,
            game_grow_rate,
            max_apple_count,
            min_apple_count,
            ticks_between_apple_spawn,
            ticks_since_last_apple_spawned: 0
        }
    }

    fn tick(&mut self, direction: &Direction) {
        /*
            1. Is move in bounds
            2. Is move into snake
            3. Is move into apple
            4. Shorten tail
            5. Put head
            6. Respawn apple
        */

        // is move in bounds
        if self.would_move_out_of_bounds(direction) {
            self.dead = true;
            return;
        }

        let new_head_pos = self.snake_head_pos.moved_direction(direction);

        let value_at_new_head_pos = self.data[new_head_pos.y][new_head_pos.x];

        // is move into body
        if value_at_new_head_pos > 0 {
            self.dead = true;
            return;
        }

        // is move into apple
        if value_at_new_head_pos == -1 {
            self.snake_saturation_len += self.game_grow_rate;
        }

        // shorten tail
        if self.snake_len < self.snake_saturation_len && ! self.grew_last_tick {
            self.snake_len += 1;
            self.grew_last_tick = true;
        } else {
            self.shorten_snake();
            self.grew_last_tick = false;
        }

        // put head
        self.data[new_head_pos.y][new_head_pos.x] = self.snake_len as i32;
        self.snake_head_pos = new_head_pos;

        // respawn apple
        let apple_count = self.count_apples();

        if apple_count < self.max_apple_count {
            self.ticks_since_last_apple_spawned += 1;

            if self.ticks_since_last_apple_spawned > self.ticks_between_apple_spawn || apple_count < self.min_apple_count {
                self.ticks_since_last_apple_spawned = 0;

                self.spawn_apple();
            }
        }

    }

    fn would_move_out_of_bounds(&self, direction: &Direction) -> bool {
        match direction {
            Direction::Up => if self.snake_head_pos.y == 0 {
                return true
            },
            Direction::Down => if self.snake_head_pos.y + 1 == self.data.len() {
                return true
            },
            Direction::Left => if self.snake_head_pos.x == 0 {
                return true
            },
            Direction::Right => if self.snake_head_pos.x + 1 == self.data[0].len() {
                return true
            }
        }
        
        false
    }

    fn shorten_snake(&mut self) {
        for row in &mut self.data {
            for col in row {
                if *col > 0 {
                    *col -= 1;
                }
            }
        }
    }

    fn count_apples(&self) -> u32 {
        let mut counter : u32 = 0;

        for row in &self.data {
            for col in row {
                if *col == -1 {
                    counter += 1;
                }
            }
        }

        counter
    }

    fn count_free(&self) -> u32 {
        let mut counter : u32 = 0;

        for row in &self.data {
            for col in row {
                if *col == 0 {
                    counter += 1;
                }
            }
        }

        counter
    }

    fn spawn_apple(&mut self) {
        let free = self.count_free();
        let pos = random::<u32>() % free;

        let mut count = 1;

        for row in &mut self.data {
            for col in row {
                if *col == 0 {
                    if count == pos {
                        *col = -1;
                        return;
                    }
                    count += 1;
                }
            }
        }
    }

    fn clear(&mut self) {
        for row in &mut self.data {
            for col in row {
                *col = 0;
            }
        }

        self.snake_head_pos = SnakeGameCord { x: (self.data[0].len() / 3) * 2, y: self.data.len() / 2 };
        self.snake_len = 0;
        self.snake_saturation_len = 3;
        self.grew_last_tick = true;
        self.dead = false;
    }
}

fn calculate_margins(width: u16, height: u16) -> Result<(u16, u16)> {
    let (s_width, s_height) = size()?;

    let width = if width + 2 > s_width {
        s_width - 2
    } else {
        width
    };
    let height = if height + 2 > s_height {
        s_height - 2
    } else {
        height
    };

    Ok(((s_width - width - 2) / 2, (s_height - height - 2) / 2))
}

fn display_game(stdout: &mut Stdout, game: &SnakeGame) -> Result<()> {
    let (margin_left, margin_top) = calculate_margins(game.data[0].len() as u16, game.data.len() as u16)?;
    
    execute!(stdout, MoveTo(margin_left, margin_top), Print(String::from("#").repeat(game.data[0].len() + 2)))?;

    for (y, row) in game.data.iter().enumerate() {
        execute!(stdout, MoveTo(margin_left, y as u16 + 1 + margin_top), Print("#"))?;

        for col in row {
            match *col {
                -1 => execute!(stdout, SetForegroundColor(Color::Red), Print("0"), SetForegroundColor(Color::White))?,
                0 => execute!(stdout, Print(" "))?,
                _ => execute!(stdout, SetForegroundColor(Color::Green), Print("+"), SetForegroundColor(Color::White))?
            }
        }

        execute!(stdout, Print("#"))?;
    }

    execute!(stdout, MoveTo(margin_left, game.data.len() as u16 + 1 + margin_top), Print(String::from("#").repeat(game.data[0].len() + 2)))?;

    Ok(())
}



// the width and height are the inner width and height
fn menue(stdout: &mut Stdout, width: u16, height: u16, title: Option<&str>, items: &[&str]) -> Result<String> {
    let mut selected = 0;

    execute!(stdout, Clear(ClearType::All))?;
    
    loop {
        let (margin_left, mut margin_top) = calculate_margins(width, height)?;
        let mut height = height;

        // lines
        let bar = String::from("#").repeat((width + 2) as usize);
        let inner_bar = format!("#{}#", String::from(" ").repeat(width as usize));

        //draw square
        execute!(
            stdout,
            MoveTo(margin_left, margin_top),
            Print(&bar),
            MoveTo(margin_left, margin_top + height + 1),
            Print(bar),
        )?;

        for y in 1..(height + 1) {
            execute!(stdout, MoveTo(margin_left, margin_top + y), Print(&inner_bar))?;
        }

        // title
        match &title {
            Some(title_text) => {
                execute!(
                    stdout, 
                    MoveTo(margin_left + (width - title_text.len() as u16) / 2 + 1, margin_top + 1), 
                    SetForegroundColor(Color::Green), 
                    Print(title_text), 
                    SetForegroundColor(Color::White)
                )?;
                height -= 1;
                margin_top += 1;
            },
            _ => {}
        }

        // draw entries
        let spacing = (height as usize - items.len()) / (items.len() + 1);

        for (i, item) in items.iter().enumerate() {
            execute!(stdout, MoveTo(margin_left + (width - item.len() as u16) / 2 + 1, margin_top + ((i as u16 + 1) * (spacing as u16)) + 1 + i as u16))?;

            if i == selected {
                execute!(stdout, SetBackgroundColor(Color::White), SetForegroundColor(Color::Black))?;
            }

            execute!(stdout, Print(item), SetBackgroundColor(Color::Black), SetForegroundColor(Color::White))?;
        }

        // process events
        match read()? {
            Event::Key(keyevent) => match keyevent.code {
                KeyCode::Down => if selected < items.len() - 1 {
                    selected += 1;
                },
                KeyCode::Up => if selected > 0 {
                    selected -= 1;
                },
                KeyCode::Enter => {
                    return Ok(String::from(items[selected]));
                },
                _ => {}
            },
            _ => {}
        }
    }
}

fn play_game(stdout: &mut Stdout, game: &mut SnakeGame) -> Result<()> {
    'retry: loop {
        let mut direction = Direction::Right;

        let mut queue = vec![];

        execute!(stdout, Clear(ClearType::All))?;

        for _ in 0..3 {
            game.tick(&direction);
        }

        while ! game.dead {
            let now = Instant::now();

            while poll(Duration::from_secs(0))? {
                
                match read()? {
                    Event::Key(key_event) => {
                        match key_event.code {
                            KeyCode::Up => queue.push(Direction::Up),
                            KeyCode::Down => queue.push(Direction::Down),
                            KeyCode::Left => queue.push(Direction::Left),
                            KeyCode::Right => queue.push(Direction::Right),
                            KeyCode::Esc => match menue(stdout, 32, 9, None, &["CONTINUE", "EXIT"])?.as_str() {
                                "CONTINUE" => {},
                                "EXIT" => break 'retry,
                                _ => {}
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }

            let mut iter = queue.iter();

            'select_next_relevant_queue_result: loop {
                match iter.next() {
                    Some(dir) => if dir.kind() != direction.kind() {
                        direction = dir.clone();
                        break 'select_next_relevant_queue_result;
                    },
                    None => {
                        break 'select_next_relevant_queue_result;
                    }
                }
            }

            let mut new_queue = vec![];

            'collect_remaining: loop {
                match iter.next() {
                    Some(dir) => new_queue.push(dir.clone()),
                    None => {
                        break 'collect_remaining;
                    }
                }
            }

            queue = new_queue;

            game.tick(&direction);
            display_game(stdout, &game)?;

            while now.elapsed() < Duration::from_millis(if direction.kind() == DirectionKind::Horizontal {
                100
            } else {
                150
            }) {}
        }

        'viewing: loop {
            match menue(stdout, 32, 8, Some(format!("You got to a length of {}", game.snake_len).as_str()), &["RETRY", "VIEW", "EXIT"])?.as_str() {
                "RETRY" => {
                    game.clear();
                    break 'viewing;
                },
                "VIEW" => {
                    display_game(stdout, game)?;
                    wait_for_any_key_press()?;
                },
                _ => break 'retry
            }
        }
    }

    Ok(())
}

fn wait_for_any_key_press() -> Result<()> {
    'wait: loop {
        match read()? {
            Event::Key(_) => break 'wait,
            _ => {}
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();

    execute!(stdout, Hide, SetBackgroundColor(Color::Black), SetForegroundColor(Color::White), EnterAlternateScreen, SetTitle("Terminal Snake"))?;

    enable_raw_mode()?;

    'application: loop {
        'selection: loop {
            let game = &mut match menue(&mut stdout, 100, 12, Some("T E R M I N A L   S N A K E"), &["CLASSIC", "MODE", "QUIT"])?.as_str() {
                "CLASSIC" => SnakeGame::create(80, 20, 1, 1, 1, 100),
                "MODE" => match menue(&mut stdout, 80, 20, None, &["FULLSCREEN", "BACK"])?.as_str() {
                    "FULLSCREEN" => {
                        let (cols, rows) = size()?;
                        SnakeGame::create((cols - 2).into(), (rows - 2).into(), 8, 8, 1, 20)
                    },
                    _ => break 'selection
                } 
                _ => break 'application
            };
            
            play_game(&mut stdout, game)?;
        }
    }

    disable_raw_mode()?;

    execute!(stdout, Show, ResetColor, LeaveAlternateScreen)?;

    Ok(())
}

/* 
    TODO
    - display mode during play
    - more modes (easy mode)
    - save highscores (specific to mode)
    - look at highscores in the main menue
    
    - windowed version
    - upload to itch.io

    Done
    - set size
    - gamemodes
    - score
    - retry
    - color
    - esc & pause menue
    - adapt to screen size
*/
