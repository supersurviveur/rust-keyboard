//! This module defines constants and structures for keyboard keys and custom key behaviors.
//! It includes predefined key codes and custom key implementations.

use core::pin;

use keyboard_macros::{config_constraints, key_alias};

use crate::{
    Keyboard, QmkKeyboard,
    keymap::{CustomKey, Key},
};

// TODO handle modifiers
pub const MODIFIER_LEFTCTRL: &Key = &Key(1);
pub const MODIFIER_LEFTSHIFT: &Key = &Key(2);
pub const MODIFIER_LEFTALT: &Key = &Key(4);
pub const MODIFIER_LEFTGUI: &Key = &Key(8);
pub const MODIFIER_RIGHTCTRL: &Key = &Key(16);
pub const MODIFIER_RIGHTSHIFT: &Key = &Key(32);
pub const MODIFIER_RIGHTALT: &Key = &Key(64);
pub const MODIFIER_RIGHTGUI: &Key = &Key(128);

pub const KC_RESERVED: &Key = &Key(0);
pub const KC_ERROR_ROLLOVER: &Key = &Key(1);
pub const KC_POST_FAIL: &Key = &Key(2);
pub const KC_ERROR_UNDEFINED: &Key = &Key(3);
pub const KC_A: &Key = &Key(4);
pub const KC_B: &Key = &Key(5);
pub const KC_C: &Key = &Key(6);
pub const KC_D: &Key = &Key(7);
pub const KC_E: &Key = &Key(8);
pub const KC_F: &Key = &Key(9);
pub const KC_G: &Key = &Key(10);
pub const KC_H: &Key = &Key(11);
pub const KC_I: &Key = &Key(12);
pub const KC_J: &Key = &Key(13);
pub const KC_K: &Key = &Key(14);
pub const KC_L: &Key = &Key(15);
pub const KC_M: &Key = &Key(16);
pub const KC_N: &Key = &Key(17);
pub const KC_O: &Key = &Key(18);
pub const KC_P: &Key = &Key(19);
pub const KC_Q: &Key = &Key(20);
pub const KC_R: &Key = &Key(21);
pub const KC_S: &Key = &Key(22);
pub const KC_T: &Key = &Key(23);
pub const KC_U: &Key = &Key(24);
pub const KC_V: &Key = &Key(25);
pub const KC_W: &Key = &Key(26);
pub const KC_X: &Key = &Key(27);
pub const KC_Y: &Key = &Key(28);
pub const KC_Z: &Key = &Key(29);
pub const KC_1: &Key = &Key(30);
pub const KC_2: &Key = &Key(31);
pub const KC_3: &Key = &Key(32);
pub const KC_4: &Key = &Key(33);
pub const KC_5: &Key = &Key(34);
pub const KC_6: &Key = &Key(35);
pub const KC_7: &Key = &Key(36);
pub const KC_8: &Key = &Key(37);
pub const KC_9: &Key = &Key(38);
pub const KC_0: &Key = &Key(39);
pub const ENTER: &Key = &Key(40);
pub const ESCAPE: &Key = &Key(41);
#[key_alias(BCKSPC)]
pub const BACKSPACE: &Key = &Key(42);
pub const TAB: &Key = &Key(43);
pub const SPACE: &Key = &Key(44);
pub const MINUS: &Key = &Key(45);
pub const EQUAL: &Key = &Key(46);
pub const OPENING_BRACKET: &Key = &Key(47);
pub const CLOSING_BRACKET: &Key = &Key(48);
#[key_alias(BCKSLS)]
pub const BACKSLASH: &Key = &Key(49);
pub const NON_US_HASHMARK: &Key = &Key(50);
#[key_alias(SMICLN)]
pub const SEMICOLON: &Key = &Key(51);
pub const APOSTROPHE: &Key = &Key(52);
pub const GRAVE_ACCENT: &Key = &Key(53);
pub const COMMA: &Key = &Key(54);
pub const DOT: &Key = &Key(55);
pub const SLASH: &Key = &Key(56);
pub const CAPS_LOCK: &Key = &Key(57);
pub const KC_F1: &Key = &Key(58);
pub const KC_F2: &Key = &Key(59);
pub const KC_F3: &Key = &Key(60);
pub const KC_F4: &Key = &Key(61);
pub const KC_F5: &Key = &Key(62);
pub const KC_F6: &Key = &Key(63);
pub const KC_F7: &Key = &Key(64);
pub const KC_F8: &Key = &Key(65);
pub const KC_F9: &Key = &Key(66);
pub const KC_F10: &Key = &Key(67);
pub const KC_F11: &Key = &Key(68);
pub const KC_F12: &Key = &Key(69);
pub const PRINT_SCREEN: &Key = &Key(70);
pub const SCROLL_LOCK: &Key = &Key(71);
pub const PAUSE: &Key = &Key(72);
pub const INSERT: &Key = &Key(73);
pub const HOME: &Key = &Key(74);
pub const PAGE_UP: &Key = &Key(75);
pub const DELETE: &Key = &Key(76);
pub const END: &Key = &Key(77);
pub const PAGE_DOWN: &Key = &Key(78);
pub const RIGHT_ARROW: &Key = &Key(79);
pub const LEFT_ARROW: &Key = &Key(80);
pub const DOWN_ARROW: &Key = &Key(81);
pub const UP_ARROW: &Key = &Key(82);
#[key_alias(NUMLCK)]
pub const NUM_LOCK: &Key = &Key(83);
pub const KEYPAD_SLASH: &Key = &Key(84);
pub const KEYPAD_ASTERISK: &Key = &Key(85);
pub const KEYPAD_MINUS: &Key = &Key(86);
pub const KEYPAD_PLUS: &Key = &Key(87);
pub const KEYPAD_ENTER: &Key = &Key(88);
pub const KEYPAD_1: &Key = &Key(89);
pub const KEYPAD_2: &Key = &Key(90);
pub const KEYPAD_3: &Key = &Key(91);
pub const KEYPAD_4: &Key = &Key(92);
pub const KEYPAD_5: &Key = &Key(93);
pub const KEYPAD_6: &Key = &Key(94);
pub const KEYPAD_7: &Key = &Key(95);
pub const KEYPAD_8: &Key = &Key(96);
pub const KEYPAD_9: &Key = &Key(97);
pub const KEYPAD_0: &Key = &Key(98);
pub const KEYPAD_DOT: &Key = &Key(99);
pub const NON_US_BACKSLASH: &Key = &Key(100);
pub const APPLICATION: &Key = &Key(101);
pub const POWER: &Key = &Key(102);
pub const KEYPAD_EQUAL_SIGN: &Key = &Key(103);
pub const KC_F13: &Key = &Key(104);
pub const KC_F14: &Key = &Key(105);
pub const KC_F15: &Key = &Key(106);
pub const KC_F16: &Key = &Key(107);
pub const KC_F17: &Key = &Key(108);
pub const KC_F18: &Key = &Key(109);
pub const KC_F19: &Key = &Key(110);
pub const KC_F20: &Key = &Key(111);
pub const KC_F21: &Key = &Key(112);
pub const KC_F22: &Key = &Key(113);
pub const KC_F23: &Key = &Key(114);
pub const KC_F24: &Key = &Key(115);
pub const EXECUTE: &Key = &Key(116);
pub const HELP: &Key = &Key(117);
pub const MENU: &Key = &Key(118);
pub const SELECT: &Key = &Key(119);
pub const STOP: &Key = &Key(120);
pub const AGAIN: &Key = &Key(121);
pub const UNDO: &Key = &Key(122);
pub const CUT: &Key = &Key(123);
pub const COPY: &Key = &Key(124);
pub const PASTE: &Key = &Key(125);
pub const FIND: &Key = &Key(126);
pub const MUTE: &Key = &Key(127);
pub const VOLUME_UP: &Key = &Key(128);
pub const VOLUME_DOWN: &Key = &Key(129);
pub const LOCKING_CAPS_LOCK: &Key = &Key(130);
pub const LOCKING_NUM_LOCK: &Key = &Key(131);
pub const LOCKING_SCROLL_LOCK: &Key = &Key(132);
pub const KEYPAD_COMMA: &Key = &Key(133);
pub const KEYPAD_EQUAL_SIGN_AS400: &Key = &Key(134);
pub const INTERNATIONAL1: &Key = &Key(135);
pub const INTERNATIONAL2: &Key = &Key(136);
pub const INTERNATIONAL3: &Key = &Key(137);
pub const INTERNATIONAL4: &Key = &Key(138);
pub const INTERNATIONAL5: &Key = &Key(139);
pub const INTERNATIONAL6: &Key = &Key(140);
pub const INTERNATIONAL7: &Key = &Key(141);
pub const INTERNATIONAL8: &Key = &Key(142);
pub const INTERNATIONAL9: &Key = &Key(143);
pub const LANG1: &Key = &Key(144);
pub const LANG2: &Key = &Key(145);
pub const LANG3: &Key = &Key(146);
pub const LANG4: &Key = &Key(147);
pub const LANG5: &Key = &Key(148);
pub const LANG6: &Key = &Key(149);
pub const LANG7: &Key = &Key(150);
pub const LANG8: &Key = &Key(151);
pub const LANG9: &Key = &Key(152);
pub const ALTERNATE_ERASE: &Key = &Key(153);
pub const SYSREQ: &Key = &Key(154);
pub const CANCEL: &Key = &Key(155);
pub const CLEAR: &Key = &Key(156);
pub const PRIOR: &Key = &Key(157);
pub const RETURN: &Key = &Key(158);
pub const SEPARATOR: &Key = &Key(159);
pub const OUT: &Key = &Key(160);
pub const OPER: &Key = &Key(161);
pub const CLEAR_AND_AGAIN: &Key = &Key(162);
pub const CRSEL_AND_PROPS: &Key = &Key(163);
pub const EXSEL: &Key = &Key(164);
pub const KEYPAD_00: &Key = &Key(176);
pub const KEYPAD_000: &Key = &Key(177);
pub const THOUSANDS_SEPARATOR: &Key = &Key(178);
pub const DECIMAL_SEPARATOR: &Key = &Key(179);
pub const CURRENCY_UNIT: &Key = &Key(180);
pub const CURRENCY_SUB_UNIT: &Key = &Key(181);
pub const KEYPAD_OPENING_PARENTHESIS: &Key = &Key(182);
pub const KEYPAD_CLOSING_PARENTHESIS: &Key = &Key(183);
pub const KEYPAD_OPENING_BRACE: &Key = &Key(184);
pub const KEYPAD_CLOSING_BRACE: &Key = &Key(185);
pub const KEYPAD_TAB: &Key = &Key(186);
pub const KEYPAD_BACKSPACE: &Key = &Key(187);
pub const KEYPAD_A: &Key = &Key(188);
pub const KEYPAD_B: &Key = &Key(189);
pub const KEYPAD_C: &Key = &Key(190);
pub const KEYPAD_D: &Key = &Key(191);
pub const KEYPAD_E: &Key = &Key(192);
pub const KEYPAD_F: &Key = &Key(193);
pub const KEYPAD_XOR: &Key = &Key(194);
pub const KEYPAD_CARET: &Key = &Key(195);
pub const KEYPAD_PERCENTAGE: &Key = &Key(196);
pub const KEYPAD_LESS_THAN_SIGN: &Key = &Key(197);
pub const KEYPAD_GREATER_THAN_SIGN: &Key = &Key(198);
pub const KEYPAD_AMP: &Key = &Key(199);
pub const KEYPAD_AMP_AMP: &Key = &Key(200);
pub const KEYPAD_PIPE: &Key = &Key(201);
pub const KEYPAD_PIPE_PIPE: &Key = &Key(202);
pub const KEYPAD_COLON: &Key = &Key(203);
pub const KEYPAD_HASHMARK: &Key = &Key(204);
pub const KEYPAD_SPACE: &Key = &Key(205);
pub const KEYPAD_AT: &Key = &Key(206);
pub const KEYPAD_EXCLAMATION_SIGN: &Key = &Key(207);
pub const KEYPAD_MEMORY_STORE: &Key = &Key(208);
pub const KEYPAD_MEMORY_RECALL: &Key = &Key(209);
pub const KEYPAD_MEMORY_CLEAR: &Key = &Key(210);
pub const KEYPAD_MEMORY_ADD: &Key = &Key(211);
pub const KEYPAD_MEMORY_SUBTRACT: &Key = &Key(212);
pub const KEYPAD_MEMORY_MULTIPLY: &Key = &Key(213);
pub const KEYPAD_MEMORY_DIVIDE: &Key = &Key(214);
pub const KEYPAD_PLUS_AND_MINUS: &Key = &Key(215);
pub const KEYPAD_CLEAR: &Key = &Key(216);
pub const KEYPAD_CLEAR_ENTRY: &Key = &Key(217);
pub const KEYPAD_BINARY: &Key = &Key(218);
pub const KEYPAD_OCTAL: &Key = &Key(219);
pub const KEYPAD_DECIMAL: &Key = &Key(220);
pub const KEYPAD_HEXADECIMAL: &Key = &Key(221);
#[key_alias(L_CTRL)]
pub const LEFT_CTRL: &Key = &Key(224);
#[key_alias(L_SHFT)]
pub const LEFT_SHIFT: &Key = &Key(225);
#[key_alias(L_ALT)]
pub const LEFT_ALT: &Key = &Key(226);
#[key_alias(L_GUI)]
pub const LEFT_GUI: &Key = &Key(227);
#[key_alias(R_CTRL)]
pub const RIGHT_CTRL: &Key = &Key(228);
#[key_alias(R_SHFT)]
pub const RIGHT_SHIFT: &Key = &Key(229);
#[key_alias(R_ALT)]
pub const RIGHT_ALT: &Key = &Key(230);
#[key_alias(R_GUI)]
pub const RIGHT_GUI: &Key = &Key(231);
pub const MEDIA_PLAY: &Key = &Key(232);
pub const MEDIA_STOP: &Key = &Key(233);
pub const MEDIA_PREVIOUS_TRACK: &Key = &Key(234);
pub const MEDIA_NEXT_TRACK: &Key = &Key(235);
pub const MEDIA_EJECT: &Key = &Key(236);
pub const MEDIA_VOLUME_UP: &Key = &Key(237);
pub const MEDIA_VOLUME_DOWN: &Key = &Key(238);
pub const MEDIA_MUTE: &Key = &Key(239);
pub const MEDIA_WWW: &Key = &Key(240);
pub const MEDIA_BACKWARD: &Key = &Key(241);
pub const MEDIA_FORWARD: &Key = &Key(242);
pub const MEDIA_CANCEL: &Key = &Key(243);
pub const MEDIA_SEARCH: &Key = &Key(244);
pub const MEDIA_SLEEP: &Key = &Key(248);
pub const MEDIA_LOCK: &Key = &Key(249);
pub const MEDIA_RELOAD: &Key = &Key(250);
pub const MEDIA_CALCULATOR: &Key = &Key(251);

/// Represents a no-operation key that does nothing when pressed.
pub struct NoOpKey;

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for NoOpKey {}

/// A constant representing a no-operation key.
pub const NO_OP: &NoOpKey = &NoOpKey;

/// Represents a key that changes the current layer up by a specified amount.
pub struct LayerUp(pub u8);

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for LayerUp {
    /// Moves the keyboard to the specified layer when the key is pressed.
    fn on_pressed(&self, keyboard: pin::Pin<&mut crate::QmkKeyboard<User>>) {
        keyboard.layer_up(self.0);
    }
}
/// Represents a key that changes the current layer up by 1.
pub const LAYUP1: &LayerUp = &LayerUp(1);

/// Represents a key that changes the current layer down by a specified amount.
pub struct LayerDown(pub u8);

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for LayerDown {
    /// Moves the keyboard to the specified layer when the key is pressed.
    fn on_pressed(&self, keyboard: pin::Pin<&mut crate::QmkKeyboard<User>>) {
        keyboard.layer_down(self.0);
    }
}

/// Represents a key that changes the current layer down by 1.
pub const LAYDW1: &LayerDown = &LayerDown(1);

/// Represents a key that transparently passes the key press to the layer above.
pub struct TransparentUp;

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for TransparentUp {
    /// Delegates the key press to the key in the layer above.
    fn complete_on_pressed(
        &self,
        mut keyboard: pin::Pin<&mut crate::QmkKeyboard<User>>,
        row: u8,
        column: u8,
    ) {
        let layer = keyboard.as_mut().get_layer_up(1);
        keyboard
            .get_key(layer, row, column)
            .complete_on_pressed(keyboard, row, column);
    }
    /// Delegates the key release to the key in the layer above.
    fn complete_on_released(
        &self,
        keyboard: pin::Pin<&mut crate::QmkKeyboard<User>>,
        row: u8,
        column: u8,
        key_actual_layer: u8,
    ) {
        let layer = key_actual_layer - 1;
        keyboard
            .get_key(layer, row, column)
            .complete_on_pressed(keyboard, row, column);
    }
}
