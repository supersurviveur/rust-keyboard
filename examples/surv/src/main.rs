#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
use avr_device::entry;
use panic_halt as _;

// =====================================================================
// 1. DESCRIPTEURS USB STANDARDS (Stockés en mémoire Flash)
// =====================================================================

const DEVICE_DESCRIPTOR: [u8; 18] = [
    18, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 8, 0x66, 0x66, // VID : 0x6666
    0x11, 0x11, // PID : 0x1111
    0x00, 0x01, 0, 0, 0, 1,
];

const HID_REPORT_DESCRIPTOR: [u8; 43] = [
    0x05, 0x01, 0x09, 0x06, 0xa1, 0x01, 0x05, 0x07, 0x19, 0xe0, 0x29, 0xe7, 0x15, 0x00, 0x25, 0x01,
    0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x95, 0x01, 0x75, 0x08, 0x81, 0x03, 0x95, 0x06, 0x75, 0x08,
    0x15, 0x00, 0x25, 0x65, 0x19, 0x00, 0x29, 0x65, 0x81, 0x00, 0xc0,
];

const CONFIG_DESCRIPTOR: [u8; 34] = [
    9, 0x02, 34, 0, 1, 1, 0, 0xA0, 50, // Configuration
    9, 0x04, 0, 0, 1, 0x03, 0x01, 0x01, 0, // Interface HID
    9, 0x21, 0x11, 0x01, 0, 1, 0x22, 63, 0, // Descripteur HID
    7, 0x05, 0x81, 0x03, 8, 0, 10, // Endpoint 1 (IN, Interrupt, 8 octets, 10ms)
];

// =====================================================================
// 2. PROGRAMME PRINCIPAL
// =====================================================================

#[entry]
fn main() -> ! {
    let dp = avr_device::atmega32u4::Peripherals::take().unwrap();

    // =========================================================
    // 1. COUPER LE WATCHDOG (Vital si un bootloader est présent)
    // =========================================================
    unsafe {
        core::arch::asm!("wdr"); // Réinitialise le timer du chien de garde
        let mcusr = 0x54 as *mut u8;
        let wdtcsr = 0x60 as *mut u8;
        core::ptr::write_volatile(mcusr, 0); // Effacer les drapeaux de reset
        // Séquence matérielle obligatoire pour déverrouiller le Watchdog
        core::ptr::write_volatile(wdtcsr, (1 << 4) | (1 << 3)); // WDCE | WDE
        core::ptr::write_volatile(wdtcsr, 0); // Désactiver complètement
    }

    // --- HORLOGE À 16 MHz ---
    unsafe {
        let clkpr = 0x61 as *mut u8;
        core::ptr::write_volatile(clkpr, 0x80);
        core::ptr::write_volatile(clkpr, 0x00);
    }

    // =========================================================
    // 2. CONFIGURER LA LED "TX" POUR LE DÉBOGAGE (Broche PD5)
    // =========================================================
    // Configure PD5 en sortie. Sur un Pro Micro, PD5 est la LED TX (souvent verte).
    // Elle s'allume si on la met à 0 (LOW). On la met à 1 pour l'éteindre au début.
    dp.PORTD.ddrd().modify(|_, w| w.pd5().set_bit());
    dp.PORTD.portd().modify(|_, w| w.pd5().set_bit());

    // --- CONFIGURATION DU BOUTON (Votre code) ---
    dp.PORTB.ddrb().modify(|_, w| w.pb0().clear_bit());
    dp.PORTB.portb().modify(|_, w| w.pb0().set_bit());

    // =========================================================
    // 3. INITIALISATION USB & PLL
    // =========================================================
    dp.USB_DEVICE.uhwcon().write(|w| w.uvrege().set_bit());
    dp.USB_DEVICE
        .usbcon()
        .write(|w| w.usbe().set_bit().frzclk().set_bit().otgpade().set_bit());

    // Essayer d'abord set_bit() pour 16MHz. Si la LED ne s'allume pas, changez en clear_bit().
    dp.PLL.pllcsr().write(|w| w.pindiv().set_bit());
    dp.PLL.pllcsr().modify(|_, w| w.plle().set_bit());

    // Boucle d'attente dangereuse (on peut rester bloqué ici)
    while dp.PLL.pllcsr().read().plock().bit_is_clear() {}

    // =========================================================
    // 4. PREUVE DE VIE
    // =========================================================
    // Si la LED s'allume, cela prouve que la PLL fonctionne et que le code n'est pas bloqué !
    dp.PORTD.portd().modify(|_, w| w.pd5().clear_bit());

    dp.USB_DEVICE.usbcon().modify(|_, w| w.frzclk().clear_bit());
    configure_endpoint(&dp.USB_DEVICE, 0, 0b00, false);
    dp.USB_DEVICE.udcon().write(|w| w.detach().clear_bit());
    let mut compteur: u32 = 0;
    let mut touche_pressee = false;
    loop {
        // =====================================================================
        // ÉTAPE 0 : GESTION DES RESETS DU BUS (La solution au -71 !)
        // =====================================================================
        if dp.USB_DEVICE.udint().read().eorsti().bit_is_set() {
            // Acquitter le reset
            dp.USB_DEVICE.udint().modify(|_, w| w.eorsti().clear_bit());
            // Le hardware vient de tuer tous les Endpoints. On rallume l'EP0 !
            configure_endpoint(&dp.USB_DEVICE, 0, 0b00, false);
            continue; // Repartir à zéro
        }

        // =====================================================================
        // ÉTAPE A : GESTION DE L'ÉNUMÉRATION (ENDPOINT 0)
        // =====================================================================
        dp.USB_DEVICE.uenum().write(|w| unsafe { w.bits(0) });

        // Si un Setup Packet a été reçu
        if dp.USB_DEVICE.ueintx().read().rxstpi().bit_is_set() {
            let mut setup = [0u8; 8];
            for i in 0..8 {
                setup[i] = dp.USB_DEVICE.uedatx().read().bits();
            }

            let req_type = setup[0];
            let request = setup[1];
            let value_l = setup[2];
            let value_h = setup[3];
            // La taille demandée par le PC (wLength)
            let length = (setup[7] as usize) << 8 | (setup[6] as usize);

            // Acquitter la réception du SETUP
            dp.USB_DEVICE.ueintx().modify(|_, w| w.rxstpi().clear_bit());

            if req_type == 0x80 && request == 0x06 {
                // GET_DESCRIPTOR
                if value_h == 0x01 {
                    // Device
                    send_descriptor(&dp.USB_DEVICE, &DEVICE_DESCRIPTOR, length);
                } else if value_h == 0x02 {
                    // Configuration
                    send_descriptor(&dp.USB_DEVICE, &CONFIG_DESCRIPTOR, length);
                }
            } else if req_type == 0x81 && request == 0x06 {
                // GET_DESCRIPTOR HID
                if value_h == 0x22 {
                    send_descriptor(&dp.USB_DEVICE, &HID_REPORT_DESCRIPTOR, length);
                }
            } else if request == 0x05 {
                // SET_ADDRESS
                // Le PC nous donne notre adresse (value_l)
                // 1. Envoyer un paquet vide pour acquitter
                dp.USB_DEVICE.ueintx().modify(|_, w| w.txini().clear_bit());
                while dp.USB_DEVICE.ueintx().read().txini().bit_is_clear() {}
                // 2. Assigner l'adresse dans le hardware
                dp.USB_DEVICE
                    .udaddr()
                    .write(|w| unsafe { w.bits((1 << 7) | value_l) });
            } else if request == 0x09 {
                // SET_CONFIGURATION
                // 1. CORRECTION : On acquitte d'abord la requête sur l'Endpoint 0
                // (Puisque UENUM vaut encore 0 à ce stade)
                dp.USB_DEVICE.ueintx().modify(|_, w| w.txini().clear_bit());

                // On attend que le paquet d'acquittement soit bien transmis au PC
                while dp.USB_DEVICE.ueintx().read().txini().bit_is_clear() {}

                // 2. Ensuite, on configure l'Endpoint 1 pour le clavier
                // (Ce qui va basculer UENUM à 1 en interne)
                configure_endpoint(&dp.USB_DEVICE, 1, 0b11, true); // EP1, Interrupt, IN
            } else {
                // STALL (Requête non gérée)
                dp.USB_DEVICE.ueconx().modify(|_, w| w.stallrq().set_bit());
            }
        }

// =====================================================================
        // ÉTAPE B : ENVOI AUTOMATIQUE DES TOUCHES (ENDPOINT 1)
        // =====================================================================
        dp.USB_DEVICE.uenum().write(|w| unsafe { w.bits(1) });

        // Si l'Endpoint 1 est actif et que le PC demande des données (TXINI = 1)
        if dp.USB_DEVICE.ueconx().read().epen().bit_is_set() && 
           dp.USB_DEVICE.ueintx().read().txini().bit_is_set() {
            
            // On incrémente notre compteur de temps
            compteur = compteur.wrapping_add(1);

            let mut report = [0u8; 8];
            let mut generer_paquet = false;

            // On simule une frappe toutes les ~60 000 interrogations
            if compteur % 60000 == 0 {
                report[2] = 0x04; // Touche 'A' enfoncée
                generer_paquet = true;
            } else if compteur % 60000 == 1 {
                report[2] = 0x00; // Touche 'A' relâchée (très important !)
                generer_paquet = true;
            }

            if generer_paquet {
                // Remplir la FIFO de l'Endpoint 1 avec le rapport de 8 octets
                for b in report.iter() {
                    dp.USB_DEVICE.uedatx().write(|w| unsafe { w.bits(*b) });
                }
                // Valider et envoyer le paquet physique
                dp.USB_DEVICE.ueintx().modify(|_, w| w.txini().clear_bit());
            } else {
                // Si aucune touche n'est pressée, on renvoie un rapport vide 
                // pour dire au PC "Rien de neuf" au lieu de bloquer la communication.
                for _ in 0..8 {
                    dp.USB_DEVICE.uedatx().write(|w| unsafe { w.bits(0) });
                }
                dp.USB_DEVICE.ueintx().modify(|_, w| w.txini().clear_bit());
            }
        }    }
}

// =====================================================================
// 3. FONCTIONS UTILITAIRES
// =====================================================================

fn configure_endpoint(usb: &avr_device::atmega32u4::USB_DEVICE, num: u8, ep_type: u8, is_in: bool) {
    usb.uenum().write(|w| unsafe { w.bits(num) });
    usb.ueconx().write(|w| w.epen().set_bit());
    usb.uecfg0x()
        .write(|w| unsafe { w.eptype().bits(ep_type).epdir().bit(is_in) });
    usb.uecfg1x()
        .write(|w| unsafe { w.epsize().bits(0).alloc().set_bit() });
}

// On ajoute le paramètre req_len
fn send_descriptor(usb: &avr_device::atmega32u4::USB_DEVICE, desc: &[u8], req_len: usize) {
    // On n'envoie jamais plus que ce qui est demandé, ni plus que la taille du descripteur
    let len = core::cmp::min(desc.len(), req_len);
    let data_to_send = &desc[..len];

    for chunk in data_to_send.chunks(8) {
        // Attendre que l'Endpoint soit prêt à émettre
        while usb.ueintx().read().txini().bit_is_clear() {}

        // Remplir la FIFO
        for b in chunk {
            usb.uedatx().write(|w| unsafe { w.bits(*b) });
        }

        // Valider l'envoi de ce morceau
        usb.ueintx().modify(|_, w| w.txini().clear_bit());
    }
}
