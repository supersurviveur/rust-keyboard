use crate::{
    Keyboard, KeyboardAuto,
    keymap::{CustomKey, Key},
};

pub const QK_LCTL: u16 = 0x0100;
pub const QK_LSFT: u16 = 0x0200;
pub const QK_LALT: u16 = 0x0400;
pub const QK_LGUI: u16 = 0x0800;
pub const QK_RMODS_MIN: u16 = 0x1000;
pub const QK_RCTL: u16 = 0x1100;
pub const QK_RSFT: u16 = 0x1200;
pub const QK_RALT: u16 = 0x1400;
pub const QK_RGUI: u16 = 0x1800;

/// Gets a keycode from its name with no path, automatically casting it to u16.
#[macro_export]
macro_rules! key {
    ($(_)*) => {
        $crate::key!(KC_NO)
    };

    ($key:ident) => {
        $crate::keys::$key as u16
    };

    ($key:expr) => {
        $key as u16
    };
}

/// Gets the uppercase equivalent of a keycode.
#[macro_export]
macro_rules! s {
    ($e:expr) => {
        (QK_LSFT | ($e as u16))
    };

    ($key:ident) => {
        (QK_LSFT | ($key as u16))
    };
}

/// MO(layer) from QMK
#[macro_export]
macro_rules! mo {
    ($layer:expr) => {
        ($crate::keys::QK_MOMENTARY as u16 | (($layer as u16) & 0x1F))
    };

    ($layer:ident) => {
        ($crate::keys::QK_MOMENTARY as u16 | (($layer as u16) & 0x1F))
    };
}

/// TG(layer) from QMK
#[macro_export]
macro_rules! tg {
    ($layer:expr) => {
        ($crate::keys::QK_TOGGLE_LAYER as u16 | (($layer as u16) & 0x1F))
    };

    ($layer:ident) => {
        ($crate::keys::QK_TOGGLE_LAYER as u16 | (($layer as u16) & 0x1F))
    };
}

/// TO(layer) from QMK
#[macro_export]
macro_rules! to {
    ($layer:expr) => {
        ($crate::keys::QK_TO as u16 | (($layer as u16) & 0x1F))
    };

    ($layer:ident) => {
        ($crate::keys::QK_TO as u16 | (($layer as u16) & 0x1F))
    };
}

#[macro_export]
macro_rules! c {
    ($key:ident) => {
        ($crate::keys::QK_LCTL | ($key as u16))
    };

    ($key:expr) => {
        ($crate::keys::QK_LCTL | ($key as u16))
    };
}

/* pub const KC_TILD: u16 = s!(key!(KC_GRV));
pub const KC_EXLM: u16 = s!(key!(KC_1));
pub const KC_AT: u16 = s!(key!(KC_2));
pub const KC_HASH: u16 = s!(key!(KC_3));
pub const KC_DLR: u16 = s!(key!(KC_4));
pub const KC_PERC: u16 = s!(key!(KC_5));
pub const KC_CIRC: u16 = s!(key!(KC_6));
pub const KC_AMPR: u16 = s!(key!(KC_7));
pub const KC_ASTR: u16 = s!(key!(KC_8));
pub const KC_LPRN: u16 = s!(key!(KC_9));
pub const KC_RPRN: u16 = s!(key!(KC_0));
pub const KC_UNDS: u16 = s!(key!(KC_MINUS));
pub const KC_PLUS: u16 = s!(key!(KC_EQUAL));
pub const KC_LCBR: u16 = s!(key!(KC_LEFT_BRACKET));
pub const KC_RCBR: u16 = s!(key!(KC_RIGHT_BRACKET));
pub const KC_PIPE: u16 = s!(key!(KC_BACKSLASH));
pub const KC_COLN: u16 = s!(key!(KC_SEMICOLON));
pub const KC_DQUO: u16 = s!(key!(KC_QUOTE));
pub const KC_LABK: u16 = s!(key!(KC_COMMA));
pub const KC_RABK: u16 = s!(key!(KC_DOT));
pub const KC_QUES: u16 = s!(key!(KC_SLASH));

pub const KC_TILDE: u16 = KC_TILD;
pub const KC_EXCLAIM: u16 = KC_EXLM;
pub const KC_DOLLAR: u16 = KC_DLR;
pub const KC_PERCENT: u16 = KC_PERC;
pub const KC_CIRCUMFLEX: u16 = KC_CIRC;
pub const KC_AMPERSAND: u16 = KC_AMPR;
pub const KC_ASTERISK: u16 = KC_ASTR;
pub const KC_LEFT_PAREN: u16 = KC_LPRN;
pub const KC_RIGHT_PAREN: u16 = KC_RPRN;
pub const KC_UNDERSCORE: u16 = KC_UNDS;
pub const KC_LEFT_CURLY_BRACE: u16 = KC_LCBR;
pub const KC_RIGHT_CURLY_BRACE: u16 = KC_RCBR;
pub const KC_COLON: u16 = KC_COLN;
pub const KC_DOUBLE_QUOTE: u16 = KC_DQUO;
pub const KC_DQT: u16 = KC_DQUO;
pub const KC_LEFT_ANGLE_BRACKET: u16 = KC_LABK;
pub const KC_LT: u16 = KC_LABK;
pub const KC_RIGHT_ANGLE_BRACKET: u16 = KC_RABK;
pub const KC_GT: u16 = KC_RABK;
pub const KC_QUESTION: u16 = KC_QUES; */

pub const KC_MODIFIER_LEFTCTRL: Key = Key(1);
pub const KC_MODIFIER_LEFTSHIFT: Key = Key(2);
pub const KC_MODIFIER_LEFTALT: Key = Key(4);
pub const KC_MODIFIER_LEFTGUI: Key = Key(8);
pub const KC_MODIFIER_RIGHTCTRL: Key = Key(16);
pub const KC_MODIFIER_RIGHTSHIFT: Key = Key(32);
pub const KC_MODIFIER_RIGHTALT: Key = Key(64);
pub const KC_MODIFIER_RIGHTGUI: Key = Key(128);

pub const KC_RESERVED: Key = Key(0);
pub const KC_ERROR_ROLLOVER: Key = Key(1);
pub const KC_POST_FAIL: Key = Key(2);
pub const KC_ERROR_UNDEFINED: Key = Key(3);
pub const KC_A: Key = Key(4);
pub const KC_B: Key = Key(5);
pub const KC_C: Key = Key(6);
pub const KC_D: Key = Key(7);
pub const KC_E: Key = Key(8);
pub const KC_F: Key = Key(9);
pub const KC_G: Key = Key(10);
pub const KC_H: Key = Key(11);
pub const KC_I: Key = Key(12);
pub const KC_J: Key = Key(13);
pub const KC_K: Key = Key(14);
pub const KC_L: Key = Key(15);
pub const KC_M: Key = Key(16);
pub const KC_N: Key = Key(17);
pub const KC_O: Key = Key(18);
pub const KC_P: Key = Key(19);
pub const KC_Q: Key = Key(20);
pub const KC_R: Key = Key(21);
pub const KC_S: Key = Key(22);
pub const KC_T: Key = Key(23);
pub const KC_U: Key = Key(24);
pub const KC_V: Key = Key(25);
pub const KC_W: Key = Key(26);
pub const KC_X: Key = Key(27);
pub const KC_Y: Key = Key(28);
pub const KC_Z: Key = Key(29);
pub const KC_1_AND_EXCLAMATION: Key = Key(30);
pub const KC_2_AND_AT: Key = Key(31);
pub const KC_3_AND_HASHMARK: Key = Key(32);
pub const KC_4_AND_DOLLAR: Key = Key(33);
pub const KC_5_AND_PERCENTAGE: Key = Key(34);
pub const KC_6_AND_CARET: Key = Key(35);
pub const KC_7_AND_AMPERSAND: Key = Key(36);
pub const KC_8_AND_ASTERISK: Key = Key(37);
pub const KC_9_AND_OPENING_PARENTHESIS: Key = Key(38);
pub const KC_0_AND_CLOSING_PARENTHESIS: Key = Key(39);
pub const KC_ENTER: Key = Key(40);
pub const KC_ESCAPE: Key = Key(41);
pub const KC_BACKSPACE: Key = Key(42);
pub const KC_TAB: Key = Key(43);
pub const KC_SPACE: Key = Key(44);
pub const KC_MINUS_AND_UNDERSCORE: Key = Key(45);
pub const KC_EQUAL_AND_PLUS: Key = Key(46);
pub const KC_OPENING_BRACKET_AND_OPENING_BRACE: Key = Key(47);
pub const KC_CLOSING_BRACKET_AND_CLOSING_BRACE: Key = Key(48);
pub const KC_BACKSLASH_AND_PIPE: Key = Key(49);
pub const KC_NON_US_HASHMARK_AND_TILDE: Key = Key(50);
pub const KC_SEMICOLON_AND_COLON: Key = Key(51);
pub const KC_APOSTROPHE_AND_QUOTE: Key = Key(52);
pub const KC_GRAVE_ACCENT_AND_TILDE: Key = Key(53);
pub const KC_COMMA_AND_LESS_THAN_SIGN: Key = Key(54);
pub const KC_DOT_AND_GREATER_THAN_SIGN: Key = Key(55);
pub const KC_SLASH_AND_QUESTION_MARK: Key = Key(56);
pub const KC_CAPS_LOCK: Key = Key(57);
pub const KC_F1: Key = Key(58);
pub const KC_F2: Key = Key(59);
pub const KC_F3: Key = Key(60);
pub const KC_F4: Key = Key(61);
pub const KC_F5: Key = Key(62);
pub const KC_F6: Key = Key(63);
pub const KC_F7: Key = Key(64);
pub const KC_F8: Key = Key(65);
pub const KC_F9: Key = Key(66);
pub const KC_F10: Key = Key(67);
pub const KC_F11: Key = Key(68);
pub const KC_F12: Key = Key(69);
pub const KC_PRINT_SCREEN: Key = Key(70);
pub const KC_SCROLL_LOCK: Key = Key(71);
pub const KC_PAUSE: Key = Key(72);
pub const KC_INSERT: Key = Key(73);
pub const KC_HOME: Key = Key(74);
pub const KC_PAGE_UP: Key = Key(75);
pub const KC_DELETE: Key = Key(76);
pub const KC_END: Key = Key(77);
pub const KC_PAGE_DOWN: Key = Key(78);
pub const KC_RIGHT_ARROW: Key = Key(79);
pub const KC_LEFT_ARROW: Key = Key(80);
pub const KC_DOWN_ARROW: Key = Key(81);
pub const KC_UP_ARROW: Key = Key(82);
pub const KC_NUM_LOCK: Key = Key(83);
pub const KC_KEYPAD_SLASH: Key = Key(84);
pub const KC_KEYPAD_ASTERISK: Key = Key(85);
pub const KC_KEYPAD_MINUS: Key = Key(86);
pub const KC_KEYPAD_PLUS: Key = Key(87);
pub const KC_KEYPAD_ENTER: Key = Key(88);
pub const KC_KEYPAD_1_AND_END: Key = Key(89);
pub const KC_KEYPAD_2_AND_DOWN_ARROW: Key = Key(90);
pub const KC_KEYPAD_3_AND_PAGE_DOWN: Key = Key(91);
pub const KC_KEYPAD_4_AND_LEFT_ARROW: Key = Key(92);
pub const KC_KEYPAD_5: Key = Key(93);
pub const KC_KEYPAD_6_AND_RIGHT_ARROW: Key = Key(94);
pub const KC_KEYPAD_7_AND_HOME: Key = Key(95);
pub const KC_KEYPAD_8_AND_UP_ARROW: Key = Key(96);
pub const KC_KEYPAD_9_AND_PAGE_UP: Key = Key(97);
pub const KC_KEYPAD_0_AND_INSERT: Key = Key(98);
pub const KC_KEYPAD_DOT_AND_DELETE: Key = Key(99);
pub const KC_NON_US_BACKSLASH_AND_PIPE: Key = Key(100);
pub const KC_APPLICATION: Key = Key(101);
pub const KC_POWER: Key = Key(102);
pub const KC_KEYPAD_EQUAL_SIGN: Key = Key(103);
pub const KC_F13: Key = Key(104);
pub const KC_F14: Key = Key(105);
pub const KC_F15: Key = Key(106);
pub const KC_F16: Key = Key(107);
pub const KC_F17: Key = Key(108);
pub const KC_F18: Key = Key(109);
pub const KC_F19: Key = Key(110);
pub const KC_F20: Key = Key(111);
pub const KC_F21: Key = Key(112);
pub const KC_F22: Key = Key(113);
pub const KC_F23: Key = Key(114);
pub const KC_F24: Key = Key(115);
pub const KC_EXECUTE: Key = Key(116);
pub const KC_HELP: Key = Key(117);
pub const KC_MENU: Key = Key(118);
pub const KC_SELECT: Key = Key(119);
pub const KC_STOP: Key = Key(120);
pub const KC_AGAIN: Key = Key(121);
pub const KC_UNDO: Key = Key(122);
pub const KC_CUT: Key = Key(123);
pub const KC_COPY: Key = Key(124);
pub const KC_PASTE: Key = Key(125);
pub const KC_FIND: Key = Key(126);
pub const KC_MUTE: Key = Key(127);
pub const KC_VOLUME_UP: Key = Key(128);
pub const KC_VOLUME_DOWN: Key = Key(129);
pub const KC_LOCKING_CAPS_LOCK: Key = Key(130);
pub const KC_LOCKING_NUM_LOCK: Key = Key(131);
pub const KC_LOCKING_SCROLL_LOCK: Key = Key(132);
pub const KC_KEYPAD_COMMA: Key = Key(133);
pub const KC_KEYPAD_EQUAL_SIGN_AS400: Key = Key(134);
pub const KC_INTERNATIONAL1: Key = Key(135);
pub const KC_INTERNATIONAL2: Key = Key(136);
pub const KC_INTERNATIONAL3: Key = Key(137);
pub const KC_INTERNATIONAL4: Key = Key(138);
pub const KC_INTERNATIONAL5: Key = Key(139);
pub const KC_INTERNATIONAL6: Key = Key(140);
pub const KC_INTERNATIONAL7: Key = Key(141);
pub const KC_INTERNATIONAL8: Key = Key(142);
pub const KC_INTERNATIONAL9: Key = Key(143);
pub const KC_LANG1: Key = Key(144);
pub const KC_LANG2: Key = Key(145);
pub const KC_LANG3: Key = Key(146);
pub const KC_LANG4: Key = Key(147);
pub const KC_LANG5: Key = Key(148);
pub const KC_LANG6: Key = Key(149);
pub const KC_LANG7: Key = Key(150);
pub const KC_LANG8: Key = Key(151);
pub const KC_LANG9: Key = Key(152);
pub const KC_ALTERNATE_ERASE: Key = Key(153);
pub const KC_SYSREQ: Key = Key(154);
pub const KC_CANCEL: Key = Key(155);
pub const KC_CLEAR: Key = Key(156);
pub const KC_PRIOR: Key = Key(157);
pub const KC_RETURN: Key = Key(158);
pub const KC_SEPARATOR: Key = Key(159);
pub const KC_OUT: Key = Key(160);
pub const KC_OPER: Key = Key(161);
pub const KC_CLEAR_AND_AGAIN: Key = Key(162);
pub const KC_CRSEL_AND_PROPS: Key = Key(163);
pub const KC_EXSEL: Key = Key(164);
pub const KC_KEYPAD_00: Key = Key(176);
pub const KC_KEYPAD_000: Key = Key(177);
pub const KC_THOUSANDS_SEPARATOR: Key = Key(178);
pub const KC_DECIMAL_SEPARATOR: Key = Key(179);
pub const KC_CURRENCY_UNIT: Key = Key(180);
pub const KC_CURRENCY_SUB_UNIT: Key = Key(181);
pub const KC_KEYPAD_OPENING_PARENTHESIS: Key = Key(182);
pub const KC_KEYPAD_CLOSING_PARENTHESIS: Key = Key(183);
pub const KC_KEYPAD_OPENING_BRACE: Key = Key(184);
pub const KC_KEYPAD_CLOSING_BRACE: Key = Key(185);
pub const KC_KEYPAD_TAB: Key = Key(186);
pub const KC_KEYPAD_BACKSPACE: Key = Key(187);
pub const KC_KEYPAD_A: Key = Key(188);
pub const KC_KEYPAD_B: Key = Key(189);
pub const KC_KEYPAD_C: Key = Key(190);
pub const KC_KEYPAD_D: Key = Key(191);
pub const KC_KEYPAD_E: Key = Key(192);
pub const KC_KEYPAD_F: Key = Key(193);
pub const KC_KEYPAD_XOR: Key = Key(194);
pub const KC_KEYPAD_CARET: Key = Key(195);
pub const KC_KEYPAD_PERCENTAGE: Key = Key(196);
pub const KC_KEYPAD_LESS_THAN_SIGN: Key = Key(197);
pub const KC_KEYPAD_GREATER_THAN_SIGN: Key = Key(198);
pub const KC_KEYPAD_AMP: Key = Key(199);
pub const KC_KEYPAD_AMP_AMP: Key = Key(200);
pub const KC_KEYPAD_PIPE: Key = Key(201);
pub const KC_KEYPAD_PIPE_PIPE: Key = Key(202);
pub const KC_KEYPAD_COLON: Key = Key(203);
pub const KC_KEYPAD_HASHMARK: Key = Key(204);
pub const KC_KEYPAD_SPACE: Key = Key(205);
pub const KC_KEYPAD_AT: Key = Key(206);
pub const KC_KEYPAD_EXCLAMATION_SIGN: Key = Key(207);
pub const KC_KEYPAD_MEMORY_STORE: Key = Key(208);
pub const KC_KEYPAD_MEMORY_RECALL: Key = Key(209);
pub const KC_KEYPAD_MEMORY_CLEAR: Key = Key(210);
pub const KC_KEYPAD_MEMORY_ADD: Key = Key(211);
pub const KC_KEYPAD_MEMORY_SUBTRACT: Key = Key(212);
pub const KC_KEYPAD_MEMORY_MULTIPLY: Key = Key(213);
pub const KC_KEYPAD_MEMORY_DIVIDE: Key = Key(214);
pub const KC_KEYPAD_PLUS_AND_MINUS: Key = Key(215);
pub const KC_KEYPAD_CLEAR: Key = Key(216);
pub const KC_KEYPAD_CLEAR_ENTRY: Key = Key(217);
pub const KC_KEYPAD_BINARY: Key = Key(218);
pub const KC_KEYPAD_OCTAL: Key = Key(219);
pub const KC_KEYPAD_DECIMAL: Key = Key(220);
pub const KC_KEYPAD_HEXADECIMAL: Key = Key(221);
pub const KC_LEFT_CONTROL: Key = Key(224);
pub const KC_LEFT_SHIFT: Key = Key(225);
pub const KC_LEFT_ALT: Key = Key(226);
pub const KC_LEFT_GUI: Key = Key(227);
pub const KC_RIGHT_CONTROL: Key = Key(228);
pub const KC_RIGHT_SHIFT: Key = Key(229);
pub const KC_RIGHT_ALT: Key = Key(230);
pub const KC_RIGHT_GUI: Key = Key(231);
pub const KC_MEDIA_PLAY: Key = Key(232);
pub const KC_MEDIA_STOP: Key = Key(233);
pub const KC_MEDIA_PREVIOUS_TRACK: Key = Key(234);
pub const KC_MEDIA_NEXT_TRACK: Key = Key(235);
pub const KC_MEDIA_EJECT: Key = Key(236);
pub const KC_MEDIA_VOLUME_UP: Key = Key(237);
pub const KC_MEDIA_VOLUME_DOWN: Key = Key(238);
pub const KC_MEDIA_MUTE: Key = Key(239);
pub const KC_MEDIA_WWW: Key = Key(240);
pub const KC_MEDIA_BACKWARD: Key = Key(241);
pub const KC_MEDIA_FORWARD: Key = Key(242);
pub const KC_MEDIA_CANCEL: Key = Key(243);
pub const KC_MEDIA_SEARCH: Key = Key(244);
pub const KC_MEDIA_SLEEP: Key = Key(248);
pub const KC_MEDIA_LOCK: Key = Key(249);
pub const KC_MEDIA_RELOAD: Key = Key(250);
pub const KC_MEDIA_CALCULATOR: Key = Key(251);

pub struct NoOpKey;

impl<User: KeyboardAuto> CustomKey<User> for NoOpKey {
    fn on_pressed(&self, _keyboard: &mut crate::QmkKeyboard<User>) {}
    fn on_released(&self, _keyboard: &mut crate::QmkKeyboard<User>) {}
}
pub const NO_OP: NoOpKey = NoOpKey;

pub struct LayerUp(pub u8);

impl<User: KeyboardAuto> CustomKey<User> for LayerUp {
    fn on_pressed(&self, keyboard: &mut crate::QmkKeyboard<User>) {
        keyboard.layer_up(self.0);
    }

    fn on_released(&self, _keyboard: &mut crate::QmkKeyboard<User>) {}
}
pub struct LayerDown(pub u8);

impl<User: KeyboardAuto> CustomKey<User> for LayerDown {
    fn on_pressed(&self, keyboard: &mut crate::QmkKeyboard<User>) {
        keyboard.layer_down(self.0);
    }

    fn on_released(&self, _keyboard: &mut crate::QmkKeyboard<User>) {}
}

pub struct TransparentUp;
impl<User: KeyboardAuto> CustomKey<User> for TransparentUp {
    fn complete_on_pressed(&self, keyboard: &mut crate::QmkKeyboard<User>, row: u8, column: u8) {
        let layer = keyboard.get_layer_up(1);
        keyboard
            .get_key(layer, row, column)
            .complete_on_pressed(keyboard, row, column);
    }
    fn on_released(&self, _keyboard: &mut crate::QmkKeyboard<User>) {}
}
