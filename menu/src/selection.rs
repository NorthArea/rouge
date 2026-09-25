/// Пункты меню. Список написан руками, а не собран из чего-либо: три строки читаются целиком.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Pong,
    Arkanoid,
    Asteroids,
    Shooter,
    Room,
    Quit,
}

pub const ITEMS: [MenuItem; 6] = [
    MenuItem::Pong,
    MenuItem::Arkanoid,
    MenuItem::Asteroids,
    MenuItem::Shooter,
    MenuItem::Room,
    MenuItem::Quit,
];

impl MenuItem {
    /// Подпись пункта в списке. Латиницей: кириллица встроенного шрифта Macroquad наблюдением не
    /// проверена, та же причина, что в подсказке Pong.
    pub fn label(self) -> &'static str {
        match self {
            MenuItem::Pong => "PONG",
            MenuItem::Arkanoid => "ARKANOID",
            MenuItem::Asteroids => "ASTEROIDS",
            MenuItem::Shooter => "SHOOTER",
            MenuItem::Room => "ROOM",
            MenuItem::Quit => "QUIT",
        }
    }
}

/// Курсор списка меню. Хранит только индекс — сам список задан константой `ITEMS`, поэтому здесь
/// нечего дублировать.
pub struct Selection {
    cursor: usize,
}

impl Selection {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn move_down(&mut self) {
        self.cursor = (self.cursor + 1) % ITEMS.len();
    }

    pub fn move_up(&mut self) {
        self.cursor = (self.cursor + ITEMS.len() - 1) % ITEMS.len();
    }

    pub fn confirm(&self) -> MenuItem {
        ITEMS[self.cursor]
    }

    /// `Esc` в меню даёт то же намерение «выйти», что и пункт `QUIT` (D-23).
    pub fn escape(&self) -> MenuItem {
        MenuItem::Quit
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_down_moves_the_cursor_to_the_next_item() {
        let mut selection = Selection::new();
        selection.move_down();

        assert_eq!(
            selection.cursor(),
            1,
            "курсор после одного move_down: {}, ожидалось 1",
            selection.cursor()
        );
    }

    #[test]
    fn move_up_moves_the_cursor_to_the_previous_item() {
        let mut selection = Selection::new();
        selection.move_down();
        selection.move_down();
        selection.move_up();

        assert_eq!(
            selection.cursor(),
            1,
            "курсор после move_down, move_down, move_up: {}, ожидалось 1",
            selection.cursor()
        );
    }

    #[test]
    fn move_down_wraps_from_the_last_item_to_the_first() {
        let mut selection = Selection::new();
        for _ in 0..(ITEMS.len() - 1) {
            selection.move_down();
        }

        selection.move_down();

        assert_eq!(
            selection.cursor(),
            0,
            "курсор с последнего пункта не завернул на первый: {}, ожидалось 0",
            selection.cursor()
        );
    }

    #[test]
    fn move_up_wraps_from_the_first_item_to_the_last() {
        let mut selection = Selection::new();
        selection.move_up();

        assert_eq!(
            selection.cursor(),
            ITEMS.len() - 1,
            "курсор с первого пункта не завернул на последний: {}, ожидалось {}",
            selection.cursor(),
            ITEMS.len() - 1
        );
    }

    #[test]
    fn confirm_returns_the_item_under_the_cursor() {
        let mut selection = Selection::new();
        selection.move_down();

        assert_eq!(
            selection.confirm(),
            MenuItem::Arkanoid,
            "подтверждение после одного move_down вернуло {:?}, ожидалось Arkanoid",
            selection.confirm()
        );
    }

    #[test]
    fn quit_confirms_to_the_intent_to_quit() {
        let mut selection = Selection::new();
        selection.move_down();
        selection.move_down();
        selection.move_down();
        selection.move_down();
        selection.move_down();

        assert_eq!(
            selection.confirm(),
            MenuItem::Quit,
            "подтверждение на пункте QUIT вернуло {:?}, ожидалось Quit",
            selection.confirm()
        );
    }

    #[test]
    fn escape_gives_the_same_intent_as_quit() {
        let selection = Selection::new();

        assert_eq!(
            selection.escape(),
            MenuItem::Quit,
            "Esc вернул {:?}, ожидалось Quit — то же намерение, что и пункт QUIT",
            selection.escape()
        );
    }

    #[test]
    fn each_item_label_matches_its_name() {
        assert_eq!(MenuItem::Pong.label(), "PONG");
        assert_eq!(MenuItem::Arkanoid.label(), "ARKANOID");
        assert_eq!(MenuItem::Asteroids.label(), "ASTEROIDS");
        assert_eq!(MenuItem::Shooter.label(), "SHOOTER");
        assert_eq!(MenuItem::Room.label(), "ROOM");
        assert_eq!(MenuItem::Quit.label(), "QUIT");
    }

    #[test]
    fn moving_down_two_items_points_at_asteroids() {
        let mut selection = Selection::new();
        selection.move_down();
        selection.move_down();

        assert_eq!(
            selection.confirm(),
            MenuItem::Asteroids,
            "подтверждение после двух move_down вернуло {:?}, ожидалось Asteroids",
            selection.confirm()
        );
    }

    #[test]
    fn moving_down_three_items_points_at_shooter() {
        let mut selection = Selection::new();
        selection.move_down();
        selection.move_down();
        selection.move_down();

        assert_eq!(
            selection.confirm(),
            MenuItem::Shooter,
            "подтверждение после трёх move_down вернуло {:?}, ожидалось Shooter",
            selection.confirm()
        );
    }

    #[test]
    fn moving_down_four_items_points_at_room() {
        let mut selection = Selection::new();
        selection.move_down();
        selection.move_down();
        selection.move_down();
        selection.move_down();

        assert_eq!(
            selection.confirm(),
            MenuItem::Room,
            "подтверждение после четырёх move_down вернуло {:?}, ожидалось Room",
            selection.confirm()
        );
    }
}
