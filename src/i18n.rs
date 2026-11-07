//! Languages: English, Tamil and Hindi. The current language is global so any part of the
//! program (window or report) can ask for words with `tr` and the name helpers below.

use crate::vedic::Graha;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Lang {
    #[default]
    En,
    Ta,
    Hi,
}

pub const LANGS: [(Lang, &str); 3] = [(Lang::En, "English"), (Lang::Ta, "தமிழ்"), (Lang::Hi, "हिन्दी")];

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    LANG.store(l as u8, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::Ta,
        2 => Lang::Hi,
        _ => Lang::En,
    }
}

/// Index into the per-language arrays below.
pub fn li() -> usize {
    lang() as usize
}

/// How rashi (sign) names are shown: regional (per language), Western (Aries…Pisces), or both.
#[derive(Clone, Copy, PartialEq)]
pub enum SignStyle {
    Regional,
    Western,
    Both,
}
static SIGN_STYLE: AtomicU8 = AtomicU8::new(0);
pub fn set_sign_style(s: SignStyle) {
    SIGN_STYLE.store(s as u8, Ordering::Relaxed);
}
fn sign_style() -> SignStyle {
    match SIGN_STYLE.load(Ordering::Relaxed) {
        1 => SignStyle::Western,
        2 => SignStyle::Both,
        _ => SignStyle::Regional,
    }
}

/// Translate a window or report string (English is the key; unknown strings stay English).
pub fn tr(s: &'static str) -> &'static str {
    if lang() == Lang::En {
        return s;
    }
    static MAP: OnceLock<HashMap<&'static str, (&'static str, &'static str)>> = OnceLock::new();
    let map = MAP.get_or_init(|| UI.iter().map(|&(e, t, h)| (e, (t, h))).collect());
    match map.get(s) {
        Some((t, h)) => {
            if lang() == Lang::Ta {
                t
            } else {
                h
            }
        }
        None => s,
    }
}

/// Translate a template and fill `{0}`, `{1}`… with `args`.
pub fn trf(template: &'static str, args: &[String]) -> String {
    fill(tr(template), args)
}

/// Translate a runtime string (used by the web chrome's `{{…}}` pass). Unknown strings stay English.
pub fn tr_dyn(s: &str) -> String {
    if lang() == Lang::En {
        return s.to_string();
    }
    static MAP: OnceLock<HashMap<&'static str, (&'static str, &'static str)>> = OnceLock::new();
    let map = MAP.get_or_init(|| UI.iter().map(|&(e, t, h)| (e, (t, h))).collect());
    match map.get(s) {
        Some((t, h)) => if lang() == Lang::Ta { t.to_string() } else { h.to_string() },
        None => s.to_string(),
    }
}

pub fn fill(template: &str, args: &[String]) -> String {
    let mut out = template.to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), a);
    }
    out
}

// ---------------------------------------------------------------- names

const SIGN_NAMES: [[&str; 12]; 3] = [
    ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"],
    ["மேஷம்", "ரிஷபம்", "மிதுனம்", "கடகம்", "சிம்மம்", "கன்னி", "துலாம்", "விருச்சிகம்", "தனுசு", "மகரம்", "கும்பம்", "மீனம்"],
    ["मेष", "वृषभ", "मिथुन", "कर्क", "सिंह", "कन्या", "तुला", "वृश्चिक", "धनु", "मकर", "कुंभ", "मीन"],
];
const SIGN_SHORT: [[&str; 12]; 3] = [
    ["Ari", "Tau", "Gem", "Can", "Leo", "Vir", "Lib", "Sco", "Sag", "Cap", "Aqu", "Pis"],
    ["மேஷ", "ரிஷ", "மிது", "கட", "சிம்", "கன்", "துலா", "விரு", "தனு", "மக", "கும்", "மீன"],
    ["मेष", "वृष", "मिथु", "कर्क", "सिंह", "कन्या", "तुला", "वृश्चि", "धनु", "मकर", "कुंभ", "मीन"],
];
const SIGN_SKT: [&str; 12] = [
    "Mesha", "Vrishabha", "Mithuna", "Karka", "Simha", "Kanya", "Tula", "Vrischika", "Dhanu", "Makara", "Kumbha", "Meena",
];
const NAK_NAMES: [[&str; 27]; 3] = [
    [
        "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra", "Punarvasu", "Pushya", "Ashlesha", "Magha",
        "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha", "Mula",
        "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishta", "Shatabhisha", "Purva Bhadrapada",
        "Uttara Bhadrapada", "Revati",
    ],
    [
        "அஸ்வினி", "பரணி", "கார்த்திகை", "ரோகிணி", "மிருகசீரிஷம்", "திருவாதிரை", "புனர்பூசம்", "பூசம்", "ஆயில்யம்", "மகம்",
        "பூரம்", "உத்திரம்", "அஸ்தம்", "சித்திரை", "சுவாதி", "விசாகம்", "அனுஷம்", "கேட்டை", "மூலம்", "பூராடம்",
        "உத்திராடம்", "திருவோணம்", "அவிட்டம்", "சதயம்", "பூரட்டாதி", "உத்திரட்டாதி", "ரேவதி",
    ],
    [
        "अश्विनी", "भरणी", "कृत्तिका", "रोहिणी", "मृगशिरा", "आर्द्रा", "पुनर्वसु", "पुष्य", "आश्लेषा", "मघा",
        "पूर्वा फाल्गुनी", "उत्तरा फाल्गुनी", "हस्त", "चित्रा", "स्वाति", "विशाखा", "अनुराधा", "ज्येष्ठा", "मूल",
        "पूर्वाषाढ़ा", "उत्तराषाढ़ा", "श्रवण", "धनिष्ठा", "शतभिषा", "पूर्वा भाद्रपद", "उत्तरा भाद्रपद", "रेवती",
    ],
];
const PLANET_NAMES: [[&str; 9]; 3] = [
    ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"],
    ["சூரியன்", "சந்திரன்", "செவ்வாய்", "புதன்", "குரு", "சுக்கிரன்", "சனி", "ராகு", "கேது"],
    ["सूर्य", "चंद्र", "मंगल", "बुध", "गुरु", "शुक्र", "शनि", "राहु", "केतु"],
];
const PLANET_SHORT: [[&str; 9]; 3] = [
    ["Su", "Mo", "Ma", "Me", "Ju", "Ve", "Sa", "Ra", "Ke"],
    ["சூ", "சந்", "செவ்", "பு", "கு", "சுக்", "சனி", "ரா", "கே"],
    ["सू", "चं", "मं", "बु", "गु", "शु", "श", "रा", "के"],
];
const TITHI_NAMES: [[&str; 16]; 3] = [
    [
        "Pratipada", "Dwitiya", "Tritiya", "Chaturthi", "Panchami", "Shashthi", "Saptami", "Ashtami", "Navami", "Dashami",
        "Ekadashi", "Dwadashi", "Trayodashi", "Chaturdashi", "Purnima", "Amavasya",
    ],
    [
        "பிரதமை", "துவிதியை", "திருதியை", "சதுர்த்தி", "பஞ்சமி", "சஷ்டி", "சப்தமி", "அஷ்டமி", "நவமி", "தசமி", "ஏகாதசி",
        "துவாதசி", "திரயோதசி", "சதுர்த்தசி", "பௌர்ணமி", "அமாவாசை",
    ],
    [
        "प्रतिपदा", "द्वितीया", "तृतीया", "चतुर्थी", "पंचमी", "षष्ठी", "सप्तमी", "अष्टमी", "नवमी", "दशमी", "एकादशी",
        "द्वादशी", "त्रयोदशी", "चतुर्दशी", "पूर्णिमा", "अमावस्या",
    ],
];
const WEEKDAY_NAMES: [[&str; 7]; 3] = [
    ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    ["ஞாயிறு", "திங்கள்", "செவ்வாய்", "புதன்", "வியாழன்", "வெள்ளி", "சனி"],
    ["रविवार", "सोमवार", "मंगलवार", "बुधवार", "गुरुवार", "शुक्रवार", "शनिवार"],
];
const YOGA_NAMES: [[&str; 27]; 3] = [
    [
        "Vishkambha", "Priti", "Ayushman", "Saubhagya", "Shobhana", "Atiganda", "Sukarma", "Dhriti", "Shula", "Ganda",
        "Vriddhi", "Dhruva", "Vyaghata", "Harshana", "Vajra", "Siddhi", "Vyatipata", "Variyan", "Parigha", "Shiva",
        "Siddha", "Sadhya", "Shubha", "Shukla", "Brahma", "Indra", "Vaidhriti",
    ],
    [
        "விஷ்கம்பம்", "ப்ரீதி", "ஆயுஷ்மான்", "சௌபாக்கியம்", "சோபனம்", "அதிகண்டம்", "சுகர்மம்", "திருதி", "சூலம்", "கண்டம்",
        "விருத்தி", "துருவம்", "வியாகாதம்", "ஹர்ஷணம்", "வஜ்ரம்", "சித்தி", "வியதீபாதம்", "வரியான்", "பரிகம்", "சிவம்",
        "சித்தம்", "சாத்தியம்", "சுபம்", "சுக்லம்", "பிராம்மம்", "ஐந்திரம்", "வைதிருதி",
    ],
    [
        "विष्कम्भ", "प्रीति", "आयुष्मान", "सौभाग्य", "शोभन", "अतिगण्ड", "सुकर्मा", "धृति", "शूल", "गण्ड", "वृद्धि",
        "ध्रुव", "व्याघात", "हर्षण", "वज्र", "सिद्धि", "व्यतीपात", "वरीयान", "परिघ", "शिव", "सिद्ध", "साध्य", "शुभ",
        "शुक्ल", "ब्रह्म", "इन्द्र", "वैधृति",
    ],
];
/// Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti, Shakuni, Chatushpada, Naga, Kimstughna.
const KARANA_NAMES: [[&str; 11]; 3] = [
    ["Bava", "Balava", "Kaulava", "Taitila", "Garaja", "Vanija", "Vishti (Bhadra)", "Shakuni", "Chatushpada", "Naga", "Kimstughna"],
    ["பவம்", "பாலவம்", "கௌலவம்", "தைதுலம்", "கரசை", "வணிசை", "பத்திரை (விஷ்டி)", "சகுனி", "சதுஷ்பாதம்", "நாகவம்", "கிம்ஸ்துக்னம்"],
    ["बव", "बालव", "कौलव", "तैतिल", "गर", "वणिज", "विष्टि (भद्रा)", "शकुनि", "चतुष्पद", "नाग", "किंस्तुघ्न"],
];
const MONTH_SHORT: [[&str; 12]; 3] = [
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
    ["ஜன", "பிப்", "மார்", "ஏப்", "மே", "ஜூன்", "ஜூலை", "ஆக", "செப்", "அக்", "நவ", "டிச"],
    ["जन", "फ़र", "मार्च", "अप्रै", "मई", "जून", "जुल", "अग", "सित", "अक्टू", "नव", "दिस"],
];

pub fn sign(i: usize) -> &'static str {
    let i = i % 12;
    match sign_style() {
        SignStyle::Western => SIGN_NAMES[0][i],
        SignStyle::Regional => SIGN_NAMES[li()][i],
        SignStyle::Both => {
            let l = li();
            if l == 0 {
                return SIGN_NAMES[0][i]; // English is already the Western name
            }
            // "Regional (Western)", e.g. "கும்பம் (Aquarius)". Built once, kept for the program's life.
            static BOTH: OnceLock<[[String; 12]; 2]> = OnceLock::new();
            let both = BOTH.get_or_init(|| {
                let mk = |lang: usize| std::array::from_fn::<String, 12, _>(|k| format!("{} ({})", SIGN_NAMES[lang][k], SIGN_NAMES[0][k]));
                [mk(1), mk(2)]
            });
            both[l - 1][i].as_str()
        }
    }
}
pub fn sign_short(i: usize) -> &'static str {
    SIGN_SHORT[li()][i % 12]
}
/// The sign with its Sanskrit name in English, the local name otherwise.
pub fn sign_long(i: usize) -> String {
    if lang() == Lang::En { format!("{} ({})", SIGN_NAMES[0][i % 12], SIGN_SKT[i % 12]) } else { sign(i).to_string() }
}
pub fn nak(i: usize) -> &'static str {
    NAK_NAMES[li()][i % 27]
}
pub fn planet(g: Graha) -> &'static str {
    PLANET_NAMES[li()][g as usize]
}
pub fn planet_short(g: Graha) -> &'static str {
    PLANET_SHORT[li()][g as usize]
}
/// Tithi 0..29 (Amavasya = 29).
pub fn tithi(t: usize) -> &'static str {
    let i = if t == 29 { 15 } else { t % 15 };
    TITHI_NAMES[li()][i]
}
pub fn weekday(w: usize) -> &'static str {
    WEEKDAY_NAMES[li()][w % 7]
}
pub fn nithya_yoga(y: usize) -> &'static str {
    YOGA_NAMES[li()][y % 27]
}
pub fn karana(k: usize) -> &'static str {
    KARANA_NAMES[li()][k % 11]
}
pub fn month_short(m: u32) -> &'static str {
    MONTH_SHORT[li()][(m as usize + 11) % 12]
}
pub fn paksha(shukla: bool) -> &'static str {
    if shukla { tr("Shukla paksha (waxing)") } else { tr("Krishna paksha (waning)") }
}

// ---------------------------------------------------------------- window and report strings

/// (English, Tamil, Hindi).
static UI: &[(&str, &str, &str)] = &[
    // calendar festival overlay
    ("Festivals", "பண்டிகைகள்", "त्योहार"),
    ("Use my location", "எனது இருப்பிடத்தைப் பயன்படுத்து", "मेरा स्थान उपयोग करें"),
    ("City", "நகரம்", "शहर"),
    ("Change city", "நகரத்தை மாற்று", "शहर बदलें"),
    ("Clear", "அழி", "साफ़ करें"),
    ("Festival dates are computed estimates and may differ by a day from local observance; Islamic dates are tabular.", "பண்டிகை தேதிகள் கணக்கிடப்பட்ட மதிப்பீடுகள்; உள்ளூர் அனுசரிப்பிலிருந்து ஒரு நாள் மாறுபடலாம்; இஸ்லாமியத் தேதிகள் அட்டவணை அடிப்படையிலானவை.", "त्योहार की तिथियाँ अनुमानित हैं और स्थानीय मान्यता से एक दिन भिन्न हो सकती हैं; इस्लामी तिथियाँ सारणीबद्ध हैं।"),
    ("Hindu", "இந்து", "हिन्दू"),
    ("Christian", "கிறிஸ்தவம்", "ईसाई"),
    ("Islamic", "இஸ்லாமியம்", "इस्लामी"),
    ("Buddhist", "பௌத்தம்", "बौद्ध"),
    ("Jain", "சமணம்", "जैन"),
    // festival names (transliterations)
    ("Diwali", "தீபாவளி", "दिवाली"),
    ("Holi", "ஹோலி", "होली"),
    ("Maha Shivaratri", "மகா சிவராத்திரி", "महाशिवरात्रि"),
    ("Rama Navami", "ராம நவமி", "राम नवमी"),
    ("Krishna Janmashtami", "கிருஷ்ண ஜென்மாஷ்டமி", "कृष्ण जन्माष्टमी"),
    ("Ganesh Chaturthi", "விநாயக சதுர்த்தி", "गणेश चतुर्थी"),
    ("Navaratri", "நவராத்திரி", "नवरात्रि"),
    ("Dussehra", "தசரா", "दशहरा"),
    ("Makar Sankranti", "மகர சங்கராந்தி", "मकर संक्रांति"),
    ("Ugadi", "உகாதி", "उगादि"),
    ("Raksha Bandhan", "ரக்ஷா பந்தன்", "रक्षा बंधन"),
    ("Vesak", "வேசாக்", "वेसाक"),
    ("Asalha Puja", "அசாள்ஹ பூஜை", "असाल्हा पूजा"),
    ("Magha Puja", "மாக பூஜை", "माघ पूजा"),
    ("Mahavir Jayanti", "மகாவீர் ஜயந்தி", "महावीर जयंती"),
    ("Paryushana", "பர்யுஷணம்", "पर्युषण"),
    ("Mahavir Nirvana", "மகாவீர் நிர்வாணம்", "महावीर निर्वाण"),
    ("Jnan Panchami", "ஞான பஞ்சமி", "ज्ञान पंचमी"),
    ("Christmas", "கிறிஸ்துமஸ்", "क्रिसमस"),
    ("Epiphany", "எபிபனி", "एपिफ़नी"),
    ("All Saints", "அனைத்து புனிதர்கள் நாள்", "सभी संत दिवस"),
    ("Easter", "ஈஸ்டர்", "ईस्टर"),
    ("Good Friday", "புனித வெள்ளி", "गुड फ़्राइडे"),
    ("Ash Wednesday", "சாம்பல் புதன்", "ऐश वेडनसडे"),
    ("Pentecost", "பெந்தெகோஸ்தே", "पेंटेकोस्ट"),
    ("Islamic New Year", "இஸ்லாமியப் புத்தாண்டு", "इस्लामी नववर्ष"),
    ("Ashura", "ஆஷூரா", "आशूरा"),
    ("Mawlid", "மவ்லித்", "मौलिद"),
    ("Ramadan", "ரமலான்", "रमज़ान"),
    ("Eid al-Fitr", "ஈதுல் பித்ர்", "ईद अल-फ़ित्र"),
    ("Eid al-Adha", "ஈதுல் அள்ஹா", "ईद अल-अज़हा"),
    // report save / reopen / refresh
    ("Download as Markdown", "மார்க்டவுனாகப் பதிவிறக்கு", "मार्कडाउन के रूप में डाउनलोड करें"),
    ("Open a saved report", "சேமித்த அறிக்கையைத் திற", "सहेजी गई रिपोर्ट खोलें"),
    ("Refresh to now", "இன்றைக்குப் புதுப்பி", "अभी तक ताज़ा करें"),
    ("Open report", "அறிக்கையைத் திற", "रिपोर्ट खोलें"),
    ("Written for the earlier date — rewrite to update", "முந்தைய தேதிக்கு எழுதப்பட்டது — புதுப்பிக்க மீண்டும் எழுதவும்", "पिछली तारीख़ के लिए लिखा गया — अद्यतन हेतु पुनः लिखें"),
    ("Rewrite this section with AI", "இந்தப் பகுதியை AI மூலம் மீண்டும் எழுது", "इस अनुभाग को AI से पुनः लिखें"),
    // top bar
    ("Vedic astrology — free, no licence", "வேத ஜோதிடம் — இலவசம், உரிமம் தேவையில்லை", "वैदिक ज्योतिष — निःशुल्क, बिना लाइसेंस"),
    ("Language:", "மொழி:", "भाषा:"),
    ("Chart style:", "சக்கர வடிவம்:", "कुंडली शैली:"),
    ("South", "தென்னிந்திய", "दक्षिण"),
    ("North", "வடஇந்திய", "उत्तर"),
    ("East", "கிழக்கிந்திய", "पूर्व"),
    ("Rahu/Ketu:", "ராகு/கேது:", "राहु/केतु:"),
    ("Mean", "சராசரி", "मध्यम"),
    ("True", "உண்மை", "स्पष्ट"),
    ("Ayanamsa:", "அயனாம்சம்:", "अयनांश:"),
    ("Chitrapaksha (Lahiri)", "சித்திரபக்ஷ (லாஹிரி)", "चित्रपक्ष (लाहिड़ी)"),
    ("Raman", "ராமன்", "रमण"),
    ("Krishnamurti (KP)", "கிருஷ்ணமூர்த்தி (KP)", "कृष्णमूर्ति (KP)"),
    ("User defined", "பயனர் வரையறை", "उपयोगकर्ता निर्धारित"),
    ("zero in", "பூஜ்ய ஆண்டு", "शून्य वर्ष"),
    ("″/yr", "″/ஆண்டு", "″/वर्ष"),
    // tabs
    ("Horoscope", "ஜாதகம்", "कुंडली"),
    ("Divisional charts", "வர்க்க சக்கரங்கள்", "वर्ग कुंडलियाँ"),
    ("Houses", "பாவங்கள்", "भाव"),
    ("Panchanga", "பஞ்சாங்கம்", "पंचांग"),
    ("Dasha periods", "தசா புக்தி", "दशा"),
    ("Yogas & Dosha", "யோகங்கள் & தோஷம்", "योग और दोष"),
    ("Ashtakavarga", "அஷ்டகவர்க்கம்", "अष्टकवर्ग"),
    ("Marriage matching", "திருமணப் பொருத்தம்", "विवाह मिलान"),
    ("Transits", "கோசாரம்", "गोचर"),
    ("Full report", "முழு அறிக்கை", "पूर्ण रिपोर्ट"),
    // status bar
    ("Ayanamsa ({0}): {1}   •   Julian day (UT): {2}", "அயனாம்சம் ({0}): {1}   •   ஜூலியன் நாள் (UT): {2}", "अयनांश ({0}): {1}   •   जूलियन दिन (UT): {2}"),
    // birth form
    ("Birth details", "பிறப்பு விவரங்கள்", "जन्म विवरण"),
    ("Name", "பெயர்", "नाम"),
    ("Gender", "பாலினம்", "लिंग"),
    ("Male", "ஆண்", "पुरुष"),
    ("Female", "பெண்", "स्त्री"),
    ("Date (D/M/Y)", "தேதி (நா/மா/ஆ)", "तिथि (दि/मा/व)"),
    ("Time (H:M:S)", "நேரம் (ம:நி:வி)", "समय (घं:मि:से)"),
    ("Place", "இடம்", "स्थान"),
    ("Place name", "இடத்தின் பெயர்", "स्थान का नाम"),
    ("Latitude (N+)", "அட்சரேகை (வ+)", "अक्षांश (उ+)"),
    ("Longitude (E+)", "தீர்க்கரேகை (கி+)", "देशांतर (पू+)"),
    ("Time zone (h)", "நேர மண்டலம் (ம)", "समय क्षेत्र (घं)"),
    ("Save profile", "சேமி", "सहेजें"),
    ("New", "புதியது", "नया"),
    ("Now", "இப்போது", "अभी"),
    ("Use the current date and time", "தற்போதைய தேதி, நேரத்தைப் பயன்படுத்து", "वर्तमान तिथि और समय लें"),
    ("Saved profiles", "சேமித்த ஜாதகங்கள்", "सहेजी गई कुंडलियाँ"),
    ("None yet. Fill in the details and press Save.", "இன்னும் இல்லை. விவரங்களை நிரப்பி சேமி அழுத்தவும்.", "अभी कोई नहीं। विवरण भरकर सहेजें दबाएँ।"),
    ("Open this profile", "இந்த ஜாதகத்தைத் திற", "यह कुंडली खोलें"),
    ("Delete", "நீக்கு", "हटाएँ"),
    ("Saved “{0}” to {1}", "“{0}” {1} இல் சேமிக்கப்பட்டது", "“{0}” {1} में सहेजा गया"),
    ("Could not save: {0}", "சேமிக்க இயலவில்லை: {0}", "सहेजा नहीं जा सका: {0}"),
    ("New person", "புதிய நபர்", "नया व्यक्ति"),
    // horoscope tab
    ("{0} — born {1} at {2}, {3}", "{0} — பிறப்பு {1}, {2}, {3}", "{0} — जन्म {1}, {2}, {3}"),
    ("Rasi chart (D1)", "ராசி சக்கரம் (D1)", "राशि कुंडली (D1)"),
    ("Navamsa chart (D9)", "நவாம்ச சக்கரம் (D9)", "नवांश कुंडली (D9)"),
    ("Bhava chart", "பாவ சக்கரம்", "भाव चलित कुंडली"),
    ("Rasi", "ராசி", "राशि"),
    ("Navamsa", "நவாம்சம்", "नवांश"),
    ("Planet positions (sidereal, {0})", "கிரக நிலைகள் (நிராயனம், {0})", "ग्रह स्थिति (निरयन, {0})"),
    ("Planet", "கிரகம்", "ग्रह"),
    ("Longitude", "பாகை", "भोगांश"),
    ("Sign", "ராசி", "राशि"),
    ("Degree", "பாகை (ராசியில்)", "अंश (राशि में)"),
    ("Star (nakshatra)", "நட்சத்திரம்", "नक्षत्र"),
    ("Pada", "பாதம்", "चरण"),
    ("Star lord", "நட்சத்திர அதிபதி", "नक्षत्र स्वामी"),
    ("House", "பாவம்", "भाव"),
    ("Strength", "பலம்", "बल"),
    ("Ascendant", "லக்னம்", "लग्न"),
    ("Asc", "லக்", "लग्न"),
    ("Exalted", "உச்சம்", "उच्च"),
    ("Debilitated", "நீசம்", "नीच"),
    ("Own sign", "ஆட்சி", "स्वराशि"),
    ("Friendly", "நட்பு", "मित्र राशि"),
    ("Neutral", "சமம்", "सम"),
    ("Enemy", "பகை", "शत्रु राशि"),
    ("Retrograde", "வக்கிரம்", "वक्री"),
    // divisional
    ("Hora", "ஹோரை", "होरा"),
    ("Drekkana", "திரேக்காணம்", "द्रेष्काण"),
    ("Chaturthamsa", "சதுர்த்தாம்சம்", "चतुर्थांश"),
    ("Saptamsa", "சப்தாம்சம்", "सप्तमांश"),
    ("Dasamsa", "தசாம்சம்", "दशमांश"),
    ("Dwadasamsa", "துவாதசாம்சம்", "द्वादशांश"),
    ("Shodasamsa", "சோடசாம்சம்", "षोडशांश"),
    ("Trimsamsa", "திரிம்சாம்சம்", "त्रिंशांश"),
    ("Shashtiamsa", "சஷ்டியாம்சம்", "षष्ट्यंश"),
    ("Signs in every divisional chart", "ஒவ்வொரு வர்க்கத்திலும் ராசிகள்", "प्रत्येक वर्ग कुंडली में राशियाँ"),
    // houses
    ("Houses (Sripati bhava)", "பாவங்கள் (ஸ்ரீபதி முறை)", "भाव (श्रीपति पद्धति)"),
    ("Each quarter between the Ascendant and Midheaven is split in three; a house begins at its sandhi.", "லக்னம் முதல் மத்திய வானம் வரை ஒவ்வொரு கால் பகுதியும் மூன்றாகப் பிரிக்கப்படுகிறது; ஒவ்வொரு பாவமும் அதன் சந்தியில் தொடங்குகிறது.", "लग्न से मध्य आकाश तक प्रत्येक चतुर्थांश तीन भागों में बँटता है; प्रत्येक भाव अपनी संधि से आरंभ होता है।"),
    ("Begins (sandhi)", "ஆரம்பம் (சந்தி)", "आरंभ (संधि)"),
    ("Middle (madhya)", "மத்தியம்", "मध्य"),
    ("Ends", "முடிவு", "अंत"),
    ("Sign of middle", "மத்திய ராசி", "मध्य की राशि"),
    ("Lord", "அதிபதி", "स्वामी"),
    ("Planets in house", "பாவத்தில் உள்ள கிரகங்கள்", "भाव में ग्रह"),
    ("Ascendant {0} {1}   •   Midheaven {2} {3}", "லக்னம் {0} {1}   •   மத்திய வானம் {2} {3}", "लग्न {0} {1}   •   मध्य आकाश {2} {3}"),
    // panchanga
    ("Panchanga at birth", "பிறப்பின் போது பஞ்சாங்கம்", "जन्म समय का पंचांग"),
    ("Daily panchanga", "தினசரி பஞ்சாங்கம்", "दैनिक पंचांग"),
    ("Date (D/M/Y):", "தேதி (நா/மா/ஆ):", "तिथि (दि/मा/व):"),
    ("at {0}", "இடம்: {0}", "स्थान: {0}"),
    ("Today", "இன்று", "आज"),
    ("Values at sunrise, with the time each one ends.", "சூரிய உதயத்தின் போதுள்ள மதிப்புகள், ஒவ்வொன்றும் முடியும் நேரத்துடன்.", "सूर्योदय के समय के मान, प्रत्येक के समाप्ति समय सहित।"),
    ("Weekday (vara)", "கிழமை (வாரம்)", "वार"),
    ("Lunar day (tithi)", "திதி", "तिथि"),
    ("Yoga", "யோகம்", "योग"),
    ("Karana", "கரணம்", "करण"),
    ("Sunrise / sunset", "சூரிய உதயம் / அஸ்தமனம்", "सूर्योदय / सूर्यास्त"),
    ("Moon sign / Sun sign", "சந்திர ராசி / சூரிய ராசி", "चंद्र राशि / सूर्य राशि"),
    ("{0} pada {1}", "{0} {1}-ஆம் பாதம்", "{0} चरण {1}"),
    ("  — until {0}", "  — {0} வரை", "  — {0} तक"),
    ("Shukla paksha (waxing)", "வளர்பிறை (சுக்ல பட்சம்)", "शुक्ल पक्ष"),
    ("Krishna paksha (waning)", "தேய்பிறை (கிருஷ்ண பட்சம்)", "कृष्ण पक्ष"),
    // dasha
    ("Vimshottari dasha", "விம்சோத்தரி தசை", "विंशोत्तरी दशा"),
    ("Birth star {0} (pada {1}), ruled by {2}. Balance of {2} dasha at birth: {3}.", "ஜன்ம நட்சத்திரம் {0} ({1}-ஆம் பாதம்), அதிபதி {2}. பிறப்பின் போது {2} தசை இருப்பு: {3}.", "जन्म नक्षत्र {0} (चरण {1}), स्वामी {2}। जन्म के समय {2} दशा शेष: {3}।"),
    ("Open a period to see its sub-periods (bhukti) and their sub-periods (antara). The running period is highlighted.", "புக்தி, அந்தரம் காண ஒரு தசையைத் திறக்கவும். நடப்புக் காலம் வண்ணத்தில் காட்டப்படும்.", "अंतर्दशा और प्रत्यंतर देखने के लिए दशा खोलें। चालू अवधि रंग से दिखाई गई है।"),
    ("{0} mahadasha   {1}  to  {2}", "{0} மகா தசை   {1} முதல் {2} வரை", "{0} महादशा   {1} से {2} तक"),
    ("{0}–{1} bhukti   {2}  to  {3}", "{0}–{1} புக்தி   {2} முதல் {3} வரை", "{0}–{1} अंतर्दशा   {2} से {3} तक"),
    ("{0} y {1} m {2} d", "{0} ஆ {1} மா {2} நா", "{0} व {1} मा {2} दि"),
    ("Dasha", "தசை", "दशा"),
    ("Bhukti", "புக்தி", "अंतर्दशा"),
    ("Antara", "அந்தரம்", "प्रत्यंतर"),
    ("Starts", "தொடக்கம்", "आरंभ"),
    ("Ends on", "முடிவு", "समाप्ति"),
    // yogas
    ("Yogas found in the chart", "ஜாதகத்தில் உள்ள யோகங்கள்", "कुंडली में योग"),
    ("None of the checked yogas are present.", "சோதித்த யோகங்கள் எதுவும் இல்லை.", "जाँचे गए योगों में से कोई उपस्थित नहीं है।"),
    ("Mars (Kuja) dosha", "செவ்வாய் தோஷம்", "मंगल दोष"),
    ("Mars is in house {0} from the Ascendant and house {1} from the Moon.", "செவ்வாய் லக்னத்திலிருந்து {0}-ஆம் இடத்திலும் சந்திரனிலிருந்து {1}-ஆம் இடத்திலும் உள்ளார்.", "मंगल लग्न से भाव {0} में और चंद्रमा से भाव {1} में है।"),
    ("Dosha is present counted from both the Ascendant and the Moon.", "லக்னம், சந்திரன் இரண்டிலிருந்தும் தோஷம் உள்ளது.", "लग्न और चंद्रमा दोनों से दोष उपस्थित है।"),
    ("Dosha is present counted from the Ascendant only.", "லக்னத்திலிருந்து மட்டும் தோஷம் உள்ளது.", "केवल लग्न से दोष उपस्थित है।"),
    ("Dosha is present counted from the Moon only.", "சந்திரனிலிருந்து மட்டும் தோஷம் உள்ளது.", "केवल चंद्रमा से दोष उपस्थित है।"),
    ("No Mars dosha (Mars is not in houses 2, 4, 7, 8 or 12).", "செவ்வாய் தோஷம் இல்லை (செவ்வாய் 2, 4, 7, 8, 12 இல் இல்லை).", "मंगल दोष नहीं है (मंगल 2, 4, 7, 8 या 12 भाव में नहीं है)।"),
    ("Kuja dosha is present with respect to the Ascendant.", "லக்னத்தைப் பொறுத்து செவ்வாய் தோஷம் உள்ளது.", "लग्न के अनुसार मंगल दोष उपस्थित है।"),
    ("No Kuja dosha with respect to the Ascendant (Mars is not in houses 1, 2, 4, 7, 8 or 12).", "லக்னத்தைப் பொறுத்து செவ்வாய் தோஷம் இல்லை (செவ்வாய் 1, 2, 4, 7, 8, 12 இல் இல்லை).", "लग्न के अनुसार मंगल दोष नहीं है (मंगल 1, 2, 4, 7, 8 या 12 भाव में नहीं है)।"),
    ("Rahu dosha", "ராகு தோஷம்", "राहु दोष"),
    ("Ketu dosha", "கேது தோஷம்", "केतु दोष"),
    ("{0} is in house {1} from the Ascendant: {0} dosha is present.", "{0} லக்னத்திலிருந்து {1}-ஆம் இடத்தில் உள்ளதால் {0} தோஷம் உள்ளது.", "{0} लग्न से भाव {1} में है: {0} दोष उपस्थित है।"),
    ("No {0} dosha: {0} is in house {1} from the Ascendant (not 1, 2, 4, 7, 8 or 12).", "{0} தோஷம் இல்லை: {0} லக்னத்திலிருந்து {1}-ஆம் இடத்தில் உள்ளார் (1, 2, 4, 7, 8, 12 அல்ல).", "{0} दोष नहीं: {0} लग्न से भाव {1} में है (1, 2, 4, 7, 8 या 12 नहीं)।"),
    ("Jupiter aspects {0}, which reduces the dosha.", "குரு {0}-ஐப் பார்ப்பதால் தோஷம் குறைகிறது.", "गुरु की {0} पर दृष्टि है, जिससे दोष कम होता है।"),
    ("Jupiter is together with {0}, which reduces the dosha.", "குரு {0}-உடன் சேர்ந்திருப்பதால் தோஷம் குறைகிறது.", "गुरु {0} के साथ है, जिससे दोष कम होता है।"),
    ("Summary", "சுருக்கம்", "सारांश"),
    ("Benefic and malefic planets", "சுப, பாப கிரகங்கள்", "शुभ और पाप ग्रह"),
    ("Remedies", "பரிகாரங்கள்", "उपाय"),
    ("Current dasha – bhukti", "நடப்பு தசை – புக்தி", "वर्तमान दशा – अंतर्दशा"),
    ("Functional benefics", "சுப அதிபதிகள்", "कारक शुभ ग्रह"),
    ("Functional malefics", "பாப அதிபதிகள்", "कारक पाप ग्रह"),
    ("Strong planets (exalted or own sign)", "வலுவான கிரகங்கள் (உச்சம், ஆட்சி)", "बलवान ग्रह (उच्च या स्वराशि)"),
    ("Weak planets (debilitated or in an enemy's sign)", "பலவீனமான கிரகங்கள் (நீசம், பகை)", "दुर्बल ग्रह (नीच या शत्रु राशि)"),
    ("Yogas", "யோகங்கள்", "योग"),
    ("Doshas", "தோஷங்கள்", "दोष"),
    ("Strongest / weakest sign (Ashtakavarga)", "வலுவான / பலவீனமான ராசி (அஷ்டகவர்க்கம்)", "सबसे बलवान / दुर्बल राशि (अष्टकवर्ग)"),
    ("Lucky day, colour and gem (from the Lagna lord)", "அதிர்ஷ்ட நாள், நிறம், ரத்தினம் (லக்னாதிபதிப்படி)", "शुभ दिन, रंग और रत्न (लग्नेश के अनुसार)"),
    ("Less favourable dasas (by birth star)", "குறைவான சாதக தசைகள் (ஜன்ம நட்சத்திரப்படி)", "कम अनुकूल दशाएँ (जन्म नक्षत्र के अनुसार)"),
    ("Benefic / malefic by lordship of houses", "பாவ ஆதிபத்யப்படி சுப / பாபர்", "भाव स्वामित्व के अनुसार शुभ / पाप"),
    ("Lordships", "ஆதிபத்யம்", "स्वामित्व"),
    ("Star remedies", "நட்சத்திரப் பரிகாரங்கள்", "नक्षत्र उपाय"),
    ("Dasa remedies", "தசா பரிகாரங்கள்", "दशा उपाय"),
    ("Dosha remedies", "தோஷப் பரிகாரங்கள்", "दोष उपाय"),
    ("Vasumathi", "வசுமதி யோகம்", "वसुमति योग"),
    ("Amala", "அமல யோகம்", "अमल योग"),
    ("Adhi", "அதி யோகம்", "अधि योग"),
    ("Sakata", "சகட யோகம்", "शकट योग"),
    ("Parivartana", "பரிவர்த்தனை யோகம்", "परिवर्तन योग"),
    ("Raja", "ராஜ யோகம்", "राज योग"),
    ("Trigraha", "திரிகிரக யோகம்", "त्रिग्रह योग"),
    ("Chaturgraha", "சதுர்கிரக யோகம்", "चतुर्ग्रह योग"),
    ("Panchagraha", "பஞ்சகிரக யோகம்", "पंचग्रह योग"),
    ("Jupiter, Venus and Mercury are in upachaya houses (3, 6, 10, 11) from the Lagna or the Moon.", "குரு, சுக்கிரன், புதன் லக்னம் அல்லது சந்திரனிலிருந்து உபசய ஸ்தானங்களில் (3, 6, 10, 11) உள்ளனர்.", "गुरु, शुक्र और बुध लग्न या चंद्रमा से उपचय भावों (3, 6, 10, 11) में हैं।"),
    ("A natural benefic occupies the 10th house from the Lagna or the Moon.", "லக்னம் அல்லது சந்திரனிலிருந்து 10-ஆம் இடத்தில் ஒரு இயற்கைச் சுபர் உள்ளார்.", "लग्न या चंद्रमा से दसवें भाव में एक नैसर्गिक शुभ ग्रह है।"),
    ("Jupiter, Venus and Mercury are in the 6th, 7th and 8th houses from the Moon.", "குரு, சுக்கிரன், புதன் சந்திரனிலிருந்து 6, 7, 8 ஆம் இடங்களில் உள்ளனர்.", "गुरु, शुक्र और बुध चंद्रमा से छठे, सातवें और आठवें भाव में हैं।"),
    ("The Moon is in house {0} from Jupiter.", "சந்திரன் குருவிலிருந்து {0}-ஆம் இடத்தில் உள்ளார்.", "चंद्रमा गुरु से भाव {0} में है।"),
    ("{0} and {1} are in each other's signs.", "{0}, {1} ஒருவர் ராசியில் மற்றவர் உள்ளனர்.", "{0} और {1} एक-दूसरे की राशि में हैं।"),
    ("{0} (lord of house {1}) and {2} (lord of house {3}) are together in {4}.", "{0} ({1}-ஆம் பாவ அதிபதி), {2} ({3}-ஆம் பாவ அதிபதி) இருவரும் {4} ராசியில் சேர்ந்துள்ளனர்.", "{0} (भाव {1} का स्वामी) और {2} (भाव {3} का स्वामी) {4} राशि में साथ हैं।"),
    ("{0} are together in house {1} ({2}).", "{0} ஆகியோர் {1}-ஆம் பாவத்தில் ({2}) சேர்ந்துள்ளனர்.", "{0} भाव {1} ({2}) में साथ हैं।"),
    ("placed in house {0}", "{0}-ஆம் பாவத்தில் உள்ளார்", "भाव {0} में स्थित"),
    ("its lord rules an adverse star (tara) from your birth star", "இதன் அதிபதி ஜன்ம நட்சத்திரத்திலிருந்து பாதகமான தாரையை ஆள்கிறார்", "इसका स्वामी जन्म नक्षत्र से प्रतिकूल तारा का स्वामी है"),
    ("Duplicate yogi", "இரண்டாம் யோகி", "द्वितीय योगी"),
    ("Starting from {0}", "{0} முதல்", "{0} से आरंभ"),
    ("Significator (karaka): {0}", "காரகர்: {0}", "कारक: {0}"),
    ("Generate", "கணி", "गणना करें"),
    ("Offset from UTC at the birth date and place", "பிறந்த நாள், இடத்தில் UTC-இலிருந்து உள்ள நேர வேறுபாடு", "जन्म तिथि और स्थान पर UTC से अंतर"),
    ("Preset place", "தயார் இடம்", "पूर्वनिर्धारित स्थान"),
    ("Birth place", "பிறந்த இடம்", "जन्म स्थान"),
    ("City, State, Country", "நகரம், மாநிலம், நாடு", "शहर, राज्य, देश"),
    ("Fetch", "தேடு", "खोजें"),
    ("Look up latitude, longitude and time zone for the place typed here", "இங்கு எழுதிய இடத்தின் அட்சரேகை, தீர்க்கரேகை, நேர மண்டலத்தைத் தேடவும்", "यहाँ लिखे स्थान का अक्षांश, देशांतर और समय क्षेत्र खोजें"),
    ("Type the birth place as City, State, Country, then press Fetch.", "பிறந்த இடத்தை நகரம், மாநிலம், நாடு என எழுதி \"தேடு\" அழுத்தவும்.", "जन्म स्थान शहर, राज्य, देश के रूप में लिखें, फिर \"खोजें\" दबाएँ।"),
    ("Looking up the place…", "இடத்தைத் தேடுகிறது…", "स्थान खोजा जा रहा है…"),
    ("Found {0}: {1}, {2}, {3} ({4})", "கண்டது {0}: {1}, {2}, {3} ({4})", "मिला {0}: {1}, {2}, {3} ({4})"),
    ("No place by that name was found. Check the spelling, or enter latitude, longitude and time zone by hand.", "அந்தப் பெயரில் இடம் கிடைக்கவில்லை. எழுத்துக்களைச் சரிபார்க்கவும், அல்லது அட்சரேகை, தீர்க்கரேகை, நேர மண்டலத்தை நேரடியாக உள்ளிடவும்.", "इस नाम का स्थान नहीं मिला। वर्तनी जाँचें, या अक्षांश, देशांतर और समय क्षेत्र स्वयं भरें।"),
    ("Place lookup failed: {0}", "இடத் தேடல் தோல்வி: {0}", "स्थान खोज विफल: {0}"),
    ("Other matches:", "பிற பொருத்தங்கள்:", "अन्य मिलान:"),
    ("Ask a question about this chart", "இந்த ஜாதகம் பற்றி ஒரு கேள்வி கேளுங்கள்", "इस कुंडली के बारे में प्रश्न पूछें"),
    ("e.g. When is a good period for my career?", "எ.கா. என் தொழிலுக்கு நல்ல காலம் எப்போது?", "उदा. मेरे करियर के लिए अच्छा समय कब है?"),
    ("Ask", "கேள்", "पूछें"),
    ("Tick “Use AI” on the left and set up the AI service in ⚙ Settings to ask questions.", "கேள்வி கேட்க, இடப்புறம் “AI” தேர்வைக் குறித்து, ⚙ அமைப்புகளில் AI சேவையை அமைக்கவும்.", "प्रश्न पूछने के लिए बाईं ओर “AI” चुनें और ⚙ सेटिंग्स में AI सेवा सेट करें।"),
    ("Passages used: {0}", "பயன்படுத்திய பகுதிகள்: {0}", "प्रयुक्त अंश: {0}"),
    ("Classical texts", "பழைய நூல்கள்", "शास्त्रीय ग्रंथ"),
    ("Download classical texts", "பழைய நூல்களைப் பதிவிறக்கு", "शास्त्रीय ग्रंथ डाउनलोड करें"),
    ("Public-domain Brihat Jataka, Saravali, Laghu Parashari, Brihat Samhita, Surya Siddhanta and more (jyotisha-corpus, CC BY 4.0) into texts/ of the knowledge folder", "பொதுப் பயன்பாட்டு பிருஹத் ஜாதகம், சாராவளி, லகு பாராசரி, பிருஹத் சம்ஹிதை, சூர்ய சித்தாந்தம் முதலியவை (jyotisha-corpus, CC BY 4.0) அறிவுக் கோப்புறையின் texts/ இல்", "सार्वजनिक डोमेन बृहत् जातक, सारावली, लघु पाराशरी, बृहत् संहिता, सूर्य सिद्धांत आदि (jyotisha-corpus, CC BY 4.0) ज्ञान फ़ोल्डर के texts/ में"),
    ("Starting…", "தொடங்குகிறது…", "आरंभ…"),
    ("Also fetch the BPHS Sanskrit text from indike.org for my personal study (never shared or packaged)", "என் தனிப்பட்ட கற்றலுக்காக indike.org-இலிருந்து பராசர ஹோரா சாஸ்திர சமஸ்கிருத உரையையும் பெறு (பகிரப்படாது, தொகுக்கப்படாது)", "मेरे निजी अध्ययन के लिए indike.org से बृहत् पाराशर होरा शास्त्र का संस्कृत पाठ भी लाएँ (साझा या पैक नहीं होगा)"),
    ("AI instructions", "AI அறிவுறுத்தல்கள்", "AI निर्देश"),
    ("Edit AI instructions", "AI அறிவுறுத்தல்களைத் திருத்து", "AI निर्देश संपादित करें"),
    ("Opens the instructions the AI follows (what to write, what never to do); your edits are used from the next request", "AI பின்பற்றும் அறிவுறுத்தல்களைத் திறக்கும் (என்ன எழுத வேண்டும், எதைச் செய்யக்கூடாது); உங்கள் திருத்தங்கள் அடுத்த கோரிக்கையிலிருந்து பயன்படும்", "AI जिन निर्देशों का पालन करता है उन्हें खोलता है (क्या लिखना है, क्या कभी नहीं करना); आपके बदलाव अगले अनुरोध से लागू होंगे"),
    ("Calendar", "நாட்காட்டி", "पंचांग कैलेंडर"),
    ("Tamil / Vedic calendar", "தமிழ் / வேத நாட்காட்டி", "तमिल / वैदिक पंचांग"),
    ("Previous month", "முந்தைய மாதம்", "पिछला महीना"),
    ("Next month", "அடுத்த மாதம்", "अगला महीना"),
    ("This month", "இந்த மாதம்", "यह महीना"),
    ("Show for the birth place", "பிறந்த இடத்திற்குக் காட்டு", "जन्म स्थान के लिए दिखाएँ"),
    ("Recalculate the month for the place in the birth details", "பிறப்பு விவரங்களிலுள்ள இடத்திற்கு மாதத்தை மீண்டும் கணி", "जन्म विवरण के स्थान के लिए महीना फिर से गिनें"),
    ("Each day shows the Tamil date, and the lunar day (tithi) and star (nakshatra) at sunrise. Pick a day for its timings.", "ஒவ்வொரு நாளும் தமிழ்த் தேதி, சூரிய உதயத்தின் திதி, நட்சத்திரத்தைக் காட்டும். நேரங்களுக்கு ஒரு நாளைத் தேர்ந்தெடுக்கவும்.", "हर दिन तमिल तिथि-दिनांक और सूर्योदय के समय की तिथि व नक्षत्र दिखाता है। समय देखने के लिए कोई दिन चुनें।"),
    ("Observance", "விசேஷம்", "व्रत/पर्व"),
    ("Brahma muhurta (auspicious)", "பிரம்ம முகூர்த்தம் (நல்லது)", "ब्रह्म मुहूर्त (शुभ)"),
    ("Abhijit muhurta (auspicious)", "அபிஜித் முகூர்த்தம் (நல்லது)", "अभिजित मुहूर्त (शुभ)"),
    ("usually avoided on Wednesdays", "புதன்கிழமைகளில் பொதுவாகத் தவிர்க்கப்படும்", "बुधवार को प्रायः टाला जाता है"),
    ("Rahu kalam (avoid)", "ராகு காலம் (தவிர்க்கவும்)", "राहु काल (टालें)"),
    ("Yamagandam (avoid)", "எமகண்டம் (தவிர்க்கவும்)", "यमगंड (टालें)"),
    ("Gulika kalam (avoid new starts)", "குளிகை காலம் (புதிய தொடக்கம் தவிர்க்கவும்)", "गुलिक काल (नए कार्य न आरंभ करें)"),
    ("Pournami (full moon)", "பௌர்ணமி", "पूर्णिमा"),
    ("{0} year", "{0} ஆண்டு", "{0} संवत्सर"),
    ("This time zone is on the other side of Greenwich from the place's longitude. Check both, unless the place is near the Greenwich meridian or the Date Line.", "இந்த நேர மண்டலம், இடத்தின் தீர்க்கரேகைக்கு மாறாக கிரீன்விச்சின் மறுபக்கம் உள்ளது. இடம் கிரீன்விச் அல்லது தேதிக்கோட்டின் அருகில் இல்லையெனில் இரண்டையும் சரிபார்க்கவும்.", "यह समय क्षेत्र स्थान के देशांतर के विपरीत ग्रीनविच के दूसरी ओर है। यदि स्थान ग्रीनविच या तिथि रेखा के पास नहीं है तो दोनों जाँचें।"),
    ("Amavasya (new moon)", "அமாவாசை", "अमावस्या"),
    ("Ekadashi", "ஏகாதசி", "एकादशी"),
    ("Pradosham", "பிரதோஷம்", "प्रदोष"),
    ("Sankatahara Chaturthi", "சங்கடஹர சதுர்த்தி", "संकष्टी चतुर्थी"),
    ("Transits are counted from your Moon sign, {0}.", "கோசாரம் உங்கள் ஜன்ம ராசியான {0} இலிருந்து கணக்கிடப்படுகிறது.", "गोचर आपकी जन्म राशि {0} से गिना जाता है।"),
    ("Sun transits", "சூரியன் கோசாரம்", "सूर्य गोचर"),
    ("Sun in house {0} from the Moon: a good month for energy, recognition and getting ahead of rivals.", "சந்திரனிலிருந்து {0}-ஆம் இடத்தில் சூரியன்: ஆற்றல், அங்கீகாரம், போட்டியாளர்களை வெல்லுதலுக்கு நல்ல மாதம்.", "चंद्रमा से भाव {0} में सूर्य: ऊर्जा, सम्मान और प्रतिद्वंद्वियों से आगे निकलने के लिए अच्छा महीना।"),
    ("Sun in house {0} from the Moon: a month to go steadily; look after your health and avoid clashes with people in authority.", "சந்திரனிலிருந்து {0}-ஆம் இடத்தில் சூரியன்: நிதானமாகச் செல்ல வேண்டிய மாதம்; உடல்நலத்தைக் கவனித்து, அதிகாரத்தில் உள்ளவர்களுடன் மோதலைத் தவிர்க்கவும்.", "चंद्रमा से भाव {0} में सूर्य: संयम से चलने का महीना; स्वास्थ्य का ध्यान रखें और अधिकारियों से टकराव से बचें।"),
    ("Settings", "அமைப்புகள்", "सेटिंग्स"),
    ("AI writer and knowledge folder", "AI எழுத்தாளர், அறிவுக் கோப்புறை", "AI लेखक और ज्ञान फ़ोल्डर"),
    ("Details changed — press Generate to calculate.", "விவரங்கள் மாறியுள்ளன — கணிக்க \"கணி\" பொத்தானை அழுத்தவும்.", "विवरण बदले हैं — गणना के लिए \"गणना करें\" दबाएँ।"),
    ("Calculate the charts, periods and readings for the details above", "மேலே உள்ள விவரங்களுக்கு சக்கரங்கள், தசைகள், பலன்களைக் கணிக்கவும்", "ऊपर के विवरण के लिए कुंडली, दशाएँ और फल की गणना करें"),
    ("Use AI for the full report", "முழு அறிக்கைக்கு AI-ஐப் பயன்படுத்து", "पूर्ण रिपोर्ट के लिए AI का उपयोग करें"),
    ("Let the AI writer expand the house and dasa readings (set it up in Settings)", "பாவ, தசைப் பலன்களை AI எழுத்தாளர் விரிவாக்கட்டும் (அமைப்புகளில் அமைக்கவும்)", "AI लेखक भाव और दशा फलों का विस्तार करे (सेटिंग्स में सेट करें)"),
    ("Set up the AI service in ⚙ Settings first.", "முதலில் ⚙ அமைப்புகளில் AI சேவையை அமைக்கவும்.", "पहले ⚙ सेटिंग्स में AI सेवा सेट करें।"),
    ("Turn the AI writer on or off with “Use AI” on the left; set it up in ⚙ Settings at the top.", "இடப்புறமுள்ள “AI” தேர்வுப்பெட்டியால் AI எழுத்தாளரை இயக்கவும்/அணைக்கவும்; மேலே உள்ள ⚙ அமைப்புகளில் அமைக்கவும்.", "बाईं ओर “AI” विकल्प से AI लेखक चालू/बंद करें; ऊपर ⚙ सेटिंग्स में सेट करें।"),
    ("Calculated for {0}", "{0} க்குக் கணிக்கப்பட்டது", "{0} के लिए गणना पूरी"),
    ("At this young age the period shows mostly through health, studies and the family's circumstances.", "இந்த இளம் வயதில் இக்காலத்தின் பலன்கள் பெரும்பாலும் உடல்நலம், படிப்பு, குடும்பச் சூழல் வழியாகவே வெளிப்படும்.", "इस छोटी आयु में इस अवधि के फल मुख्यतः स्वास्थ्य, पढ़ाई और परिवार की परिस्थितियों के रूप में दिखेंगे।"),
    ("At this age the period shows mostly through education, friendships and first choices about a career.", "இந்த வயதில் இக்காலத்தின் பலன்கள் பெரும்பாலும் கல்வி, நட்பு, தொழில் பற்றிய முதல் முடிவுகள் வழியாக வெளிப்படும்.", "इस आयु में इस अवधि के फल मुख्यतः शिक्षा, मित्रता और करियर के पहले निर्णयों के रूप में दिखेंगे।"),
    ("At this stage of life the period shows mostly through career, relationships, income and building a home.", "வாழ்க்கையின் இந்தக் கட்டத்தில் இக்காலத்தின் பலன்கள் பெரும்பாலும் தொழில், உறவுகள், வருமானம், வீடு அமைத்தல் வழியாக வெளிப்படும்.", "जीवन के इस चरण में इस अवधि के फल मुख्यतः करियर, संबंधों, आय और घर बसाने के रूप में दिखेंगे।"),
    ("At this stage of life the period shows mostly through family responsibilities, wealth, status and the children's progress.", "வாழ்க்கையின் இந்தக் கட்டத்தில் இக்காலத்தின் பலன்கள் பெரும்பாலும் குடும்பப் பொறுப்புகள், செல்வம், அந்தஸ்து, குழந்தைகளின் முன்னேற்றம் வழியாக வெளிப்படும்.", "जीवन के इस चरण में इस अवधि के फल मुख्यतः पारिवारिक दायित्वों, धन, प्रतिष्ठा और संतान की प्रगति के रूप में दिखेंगे।"),
    ("At this stage of life the period shows mostly through health, the family's support, peace of mind and spiritual interests.", "வாழ்க்கையின் இந்தக் கட்டத்தில் இக்காலத்தின் பலன்கள் பெரும்பாலும் உடல்நலம், குடும்பத்தின் ஆதரவு, மன அமைதி, ஆன்மிக ஈடுபாடு வழியாக வெளிப்படும்.", "जीवन के इस चरण में इस अवधि के फल मुख्यतः स्वास्थ्य, परिवार के सहयोग, मन की शांति और आध्यात्मिक रुचियों के रूप में दिखेंगे।"),
    ("kendra (angular)", "கேந்திரம்", "केंद्र"),
    ("trikona (trine)", "திரிகோணம்", "त्रिकोण"),
    ("dusthana (difficult)", "துஸ்தானம்", "दुःस्थान"),
    ("upachaya (growing)", "உபஜயம்", "उपचय"),
    ("maraka", "மாரகம்", "मारक"),
    ("panaphara (succeeding)", "பணபரம்", "पणफर"),
    ("apoklima (cadent)", "ஆபோக்லிமம்", "आपोक्लिम"),
    ("Knowledge base: {0} rules, {1} Sanskrit glossary entries, {2} passages the AI writer can search.", "அறிவுத் தளம்: {0} விதிகள், {1} சமஸ்கிருதச் சொற்கள், AI எழுத்தாளர் தேடக்கூடிய {2} பகுதிகள்.", "ज्ञान आधार: {0} नियम, {1} संस्कृत शब्दावली प्रविष्टियाँ, {2} अंश जिन्हें AI लेखक खोज सकता है।"),
    ("AI writer: on ({0})", "AI எழுத்தாளர்: இயக்கத்தில் ({0})", "AI लेखक: चालू ({0})"),
    ("AI writer: off — readings come from the rules only.", "AI எழுத்தாளர்: அணைப்பில் — பலன்கள் விதிகளிலிருந்து மட்டும்.", "AI लेखक: बंद — फल केवल नियमों से।"),
    ("{0} shares a house with {1}, which it does not get along with, so part of the good of this period can be held back by friction.", "{0} தனக்குப் பகையான {1} உடன் ஒரே பாவத்தில் இருப்பதால், இந்தக் காலத்தின் நன்மைகளில் ஒரு பகுதி உரசல்களால் தடைபடலாம்.", "{0} अपने शत्रु {1} के साथ एक ही भाव में है, इसलिए इस अवधि के कुछ शुभ फल टकराव के कारण रुक सकते हैं।"),
    ("The Moon is joined by {0}; give your peace of mind and emotional balance extra care in this period.", "சந்திரனுடன் {0} சேர்ந்திருப்பதால், இந்தக் காலத்தில் மன அமைதிக்கும் உணர்ச்சி சமநிலைக்கும் கூடுதல் கவனம் தேவை.", "चंद्रमा के साथ {0} है; इस अवधि में मन की शांति और भावनात्मक संतुलन पर विशेष ध्यान दें।"),
    ("{0} Dasa", "{0} தசை", "{0} दशा"),
    ("This period also brings out the {0} yoga in your chart.", "இந்தக் காலம் உங்கள் ஜாதகத்திலுள்ள {0} யோகத்தின் பலனையும் வெளிப்படுத்தும்.", "यह अवधि आपकी कुंडली के {0} योग का फल भी प्रकट करेगी।"),
    ("Rule {0} — {1}", "விதி {0} — {1}", "नियम {0} — {1}"),
    ("paraphrased; not yet checked against a printed edition", "சுருக்கமாக மீண்டும் எழுதப்பட்டது; அச்சுப் பதிப்புடன் இன்னும் சரிபார்க்கப்படவில்லை", "सरल शब्दों में पुनर्लिखित; अभी मुद्रित संस्करण से मिलान नहीं हुआ"),
    ("AI-assisted reading", "AI உதவியுடன் எழுதிய பலன்", "AI की सहायता से लिखा फल"),
    ("Written by {0} from the rules above and the knowledge folder; check it against the rule-based text.", "மேலே உள்ள விதிகள், அறிவுக் கோப்புறையிலிருந்து {0} எழுதியது; விதி அடிப்படையிலான உரையுடன் ஒப்பிட்டுப் பார்க்கவும்.", "ऊपर के नियमों और ज्ञान फ़ोल्डर से {0} द्वारा लिखा गया; नियम-आधारित पाठ से मिलान करें।"),
    ("AI writer (optional)", "AI எழுத்தாளர் (விருப்பத்தேர்வு)", "AI लेखक (वैकल्पिक)"),
    ("The rule engine writes every reading by itself. If you add an OpenAI-compatible service here, it can also expand the house and dasa readings, using only the chart facts, the rules and passages from your knowledge folder.", "விதி இயந்திரமே எல்லாப் பலன்களையும் எழுதும். இங்கு OpenAI-இணக்கச் சேவையைச் சேர்த்தால், ஜாதக விவரங்கள், விதிகள், உங்கள் அறிவுக் கோப்புறைப் பகுதிகளை மட்டும் கொண்டு பாவ, தசைப் பலன்களை விரிவாக்கும்.", "नियम इंजन हर फल स्वयं लिखता है। यहाँ OpenAI-संगत सेवा जोड़ने पर वह केवल कुंडली तथ्यों, नियमों और आपके ज्ञान फ़ोल्डर के अंशों से भाव और दशा फलों का विस्तार भी कर सकती है।"),
    ("Use the AI writer in the full report", "முழு அறிக்கையில் AI எழுத்தாளரைப் பயன்படுத்து", "पूर्ण रिपोर्ट में AI लेखक का उपयोग करें"),
    ("Service address", "சேவை முகவரி", "सेवा पता"),
    ("API key (token)", "API சாவி (டோக்கன்)", "API कुंजी (टोकन)"),
    ("Model", "மாதிரி", "मॉडल"),
    ("User id (optional)", "பயனர் அடையாளம் (விருப்பம்)", "उपयोगकर्ता आईडी (वैकल्पिक)"),
    ("Organisation (optional)", "நிறுவனம் (விருப்பம்)", "संगठन (वैकल्पिक)"),
    ("Temperature", "வெப்பநிலை", "तापमान"),
    ("Longest answer (tokens)", "அதிகபட்ச பதில் (டோக்கன்கள்)", "अधिकतम उत्तर (टोकन)"),
    ("Knowledge folder", "அறிவுக் கோப்புறை", "ज्ञान फ़ोल्डर"),
    ("Passages per section", "ஒரு பகுதிக்கான பத்திகள்", "प्रति खंड अंश"),
    ("Open folder", "கோப்புறையைத் திற", "फ़ोल्डर खोलें"),
    ("Test connection", "இணைப்பைச் சோதி", "कनेक्शन जाँचें"),
    ("Testing…", "சோதிக்கிறது…", "जाँच हो रही है…"),
    ("Connection works: {0}", "இணைப்பு வேலை செய்கிறது: {0}", "कनेक्शन ठीक है: {0}"),
    ("AI writer error: {0}", "AI எழுத்தாளர் பிழை: {0}", "AI लेखक त्रुटि: {0}"),
    ("AI writer: section {0} of {1}…", "AI எழுத்தாளர்: பகுதி {0} / {1}…", "AI लेखक: खंड {0} / {1}…"),
    ("Your key is kept only in this computer's settings file and sent only to the address above.", "உங்கள் சாவி இந்தக் கணினியின் அமைப்புக் கோப்பில் மட்டும் வைக்கப்பட்டு மேலே உள்ள முகவரிக்கு மட்டுமே அனுப்பப்படும்.", "आपकी कुंजी केवल इस कंप्यूटर की सेटिंग फ़ाइल में रहती है और केवल ऊपर के पते पर भेजी जाती है।"),
    ("Calculation:", "கணிப்பு:", "गणना:"),
    ("Swiss Ephemeris", "சுவிஸ் எபிமெரிஸ்", "स्विस एफ़ेमेरिस"),
    ("Built-in formulas", "உள்ளமைந்த சூத்திரங்கள்", "अंतर्निहित सूत्र"),
    ("Planet strength by sign", "ராசி அடிப்படையில் கிரக பலம்", "राशि के अनुसार ग्रह बल"),
    ("Rahu–Ketu (Sarpa) dosha", "ராகு–கேது (சர்ப்ப) தோஷம்", "राहु–केतु (सर्प) दोष"),
    ("Rahu or Ketu is in house {0} from the Ascendant: Rahu–Ketu dosha is present.", "ராகு அல்லது கேது லக்னத்திலிருந்து {0}-ஆம் இடத்தில் உள்ளதால் ராகு–கேது தோஷம் உள்ளது.", "राहु या केतु लग्न से भाव {0} में है: राहु–केतु दोष उपस्थित है।"),
    ("Rahu and Ketu are not in houses 1, 2, 7 or 8 from the Ascendant: no Rahu–Ketu dosha.", "ராகு, கேது லக்னத்திலிருந்து 1, 2, 7, 8 இல் இல்லை: ராகு–கேது தோஷம் இல்லை.", "राहु और केतु लग्न से 1, 2, 7 या 8 भाव में नहीं हैं: राहु–केतु दोष नहीं है।"),
    // yoga names and reasons
    ("Gaja Kesari", "கஜகேசரி யோகம்", "गजकेसरी योग"),
    ("Budha Aditya", "புதாதித்ய யோகம்", "बुधादित्य योग"),
    ("Chandra Mangala", "சந்திர மங்கள யோகம்", "चंद्र-मंगल योग"),
    ("Ruchaka", "ருசக யோகம்", "रुचक योग"),
    ("Bhadra", "பத்ர யோகம்", "भद्र योग"),
    ("Hamsa", "ஹம்ச யோகம்", "हंस योग"),
    ("Malavya", "மாளவ்ய யோகம்", "मालव्य योग"),
    ("Sasa", "சச யோகம்", "शश योग"),
    ("Durudhara", "துருதுரா யோகம்", "दुरुधरा योग"),
    ("Sunapha", "சுனபா யோகம்", "सुनफा योग"),
    ("Anapha", "அனபா யோகம்", "अनफा योग"),
    ("Kemadruma", "கேமத்ரும யோகம்", "केमद्रुम योग"),
    ("Vesi", "வேசி யோகம்", "वेशि योग"),
    ("Kala Sarpa", "கால சர்ப்ப யோகம்", "कालसर्प योग"),
    ("Jupiter is in a kendra (1, 4, 7, 10) from the Moon.", "குரு சந்திரனிலிருந்து கேந்திரத்தில் (1, 4, 7, 10) உள்ளார்.", "गुरु चंद्रमा से केंद्र (1, 4, 7, 10) में हैं।"),
    ("Sun and Mercury together in {0}.", "சூரியனும் புதனும் {0} ராசியில் சேர்ந்துள்ளனர்.", "सूर्य और बुध {0} राशि में एक साथ हैं।"),
    ("Moon and Mars together in {0}.", "சந்திரனும் செவ்வாயும் {0} ராசியில் சேர்ந்துள்ளனர்.", "चंद्र और मंगल {0} राशि में एक साथ हैं।"),
    ("{0} is {1} in a kendra from the ascendant (Pancha Mahapurusha).", "{0} லக்னத்திலிருந்து கேந்திரத்தில் {1} நிலையில் உள்ளார் (பஞ்ச மகாபுருஷ யோகம்).", "{0} लग्न से केंद्र में {1} स्थिति में है (पंच महापुरुष योग)।"),
    ("Planets on both sides of the Moon (2nd and 12th).", "சந்திரனின் இருபுறமும் (2, 12) கிரகங்கள் உள்ளன.", "चंद्रमा के दोनों ओर (2रे और 12वें) ग्रह हैं।"),
    ("A planet in the 2nd from the Moon.", "சந்திரனுக்கு 2-ஆம் இடத்தில் ஒரு கிரகம் உள்ளது.", "चंद्रमा से दूसरे भाव में ग्रह है।"),
    ("A planet in the 12th from the Moon.", "சந்திரனுக்கு 12-ஆம் இடத்தில் ஒரு கிரகம் உள்ளது.", "चंद्रमा से बारहवें भाव में ग्रह है।"),
    ("No planets beside the Moon and none in kendras.", "சந்திரனின் இருபுறமும் கேந்திரங்களிலும் கிரகங்கள் இல்லை.", "चंद्रमा के दोनों ओर और केंद्रों में कोई ग्रह नहीं है।"),
    ("A planet (not the Moon) in the 2nd from the Sun.", "சூரியனுக்கு 2-ஆம் இடத்தில் ஒரு கிரகம் (சந்திரன் அல்லாத) உள்ளது.", "सूर्य से दूसरे भाव में (चंद्रमा के अतिरिक्त) ग्रह है।"),
    ("All seven planets lie on one side of the Rahu-Ketu axis.", "ஏழு கிரகங்களும் ராகு-கேது அச்சின் ஒரு பக்கத்தில் உள்ளன.", "सातों ग्रह राहु-केतु अक्ष के एक ओर हैं।"),
    // matching
    ("Marriage matching (Ashtakoota, 36 points)", "திருமணப் பொருத்தம் (அஷ்டகூடம், 36 புள்ளிகள்)", "विवाह मिलान (अष्टकूट, 36 गुण)"),
    ("Partner from saved profiles:", "சேமித்தவற்றிலிருந்து துணை:", "सहेजी गई कुंडलियों से साथी:"),
    ("choose…", "தேர்வு செய்க…", "चुनें…"),
    ("{0} (main profile)", "{0} (முதன்மை ஜாதகம்)", "{0} (मुख्य कुंडली)"),
    ("Moon in {0}, star {1}", "சந்திரன் {0} ராசியில், நட்சத்திரம் {1}", "चंद्रमा {0} राशि में, नक्षत्र {1}"),
    ("Partner", "துணை", "साथी"),
    ("Koota", "பொருத்தம்", "कूट"),
    ("Points", "புள்ளிகள்", "गुण"),
    ("Out of", "மொத்தம்", "अधिकतम"),
    ("Boy / Girl", "ஆண் / பெண்", "वर / कन्या"),
    ("Total", "மொத்தம்", "कुल"),
    ("Excellent match", "மிகச் சிறந்த பொருத்தம்", "उत्तम मिलान"),
    ("Acceptable match (18 or more)", "ஏற்கத்தக்க பொருத்தம் (18 அல்லது அதற்கு மேல்)", "स्वीकार्य मिलान (18 या अधिक)"),
    ("Below the usual minimum of 18", "வழக்கமான குறைந்தபட்சமான 18-க்குக் கீழ்", "सामान्य न्यूनतम 18 से कम"),
    ("Mars dosha — {0}: {1}; partner: {2}.", "செவ்வாய் தோஷம் — {0}: {1}; துணை: {2}.", "मंगल दोष — {0}: {1}; साथी: {2}।"),
    ("present", "உள்ளது", "है"),
    ("absent", "இல்லை", "नहीं है"),
    ("Varna", "வர்ணம்", "वर्ण"),
    ("Vashya", "வசியம்", "वश्य"),
    ("Tara (Dina)", "தினம் (தாரை)", "तारा"),
    ("Yoni", "யோனி", "योनि"),
    ("Graha Maitri", "கிரக மைத்ரி", "ग्रह मैत्री"),
    ("Gana", "கணம்", "गण"),
    ("Bhakoot (Rasi)", "ராசி (பகூட்)", "भकूट"),
    ("Nadi", "நாடி", "नाड़ी"),
    ("counted both ways from {0} and {1}", "{0}, {1} இரண்டிலிருந்தும் எண்ணப்பட்டது", "{0} और {1} दोनों ओर से गिना गया"),
    ("{0} / {1} ({2}/{3} apart)", "{0} / {1} ({2}/{3} இடைவெளி)", "{0} / {1} ({2}/{3} दूरी)"),
    ("Shudra", "சூத்திரர்", "शूद्र"),
    ("Vaishya", "வைசியர்", "वैश्य"),
    ("Kshatriya", "சத்திரியர்", "क्षत्रिय"),
    ("Brahmin", "பிராமணர்", "ब्राह्मण"),
    ("Chatushpada", "சதுஷ்பாதம்", "चतुष्पद"),
    ("Manava", "மனிதம்", "मानव"),
    ("Jalachara", "ஜலசரம்", "जलचर"),
    ("Vanachara", "வனசரம்", "वनचर"),
    ("Keeta", "கீடம்", "कीट"),
    ("Horse", "குதிரை", "घोड़ा"),
    ("Elephant", "யானை", "हाथी"),
    ("Sheep", "ஆடு", "भेड़"),
    ("Serpent", "பாம்பு", "सर्प"),
    ("Dog", "நாய்", "कुत्ता"),
    ("Cat", "பூனை", "बिल्ली"),
    ("Rat", "எலி", "चूहा"),
    ("Cow", "பசு", "गाय"),
    ("Buffalo", "எருமை", "भैंस"),
    ("Tiger", "புலி", "बाघ"),
    ("Deer", "மான்", "हिरण"),
    ("Monkey", "குரங்கு", "बंदर"),
    ("Mongoose", "கீரி", "नेवला"),
    ("Lion", "சிங்கம்", "सिंह"),
    ("Deva", "தேவ கணம்", "देव गण"),
    ("Manushya", "மனித கணம்", "मनुष्य गण"),
    ("Rakshasa", "ராட்சச கணம்", "राक्षस गण"),
    ("Adi (Vata)", "ஆதி (வாதம்)", "आदि (वात)"),
    ("Madhya (Pitta)", "மத்திய (பித்தம்)", "मध्य (पित्त)"),
    ("Antya (Kapha)", "அந்திய (கபம்)", "अंत्य (कफ)"),
    // transits
    ("Planets now ({0} at {1})", "தற்போதைய கிரக நிலை ({0}, {1})", "वर्तमान ग्रह स्थिति ({0}, {1})"),
    ("natal Mo", "ஜன்ம சந்", "जन्म चं"),
    ("Sign now", "தற்போதைய ராசி", "वर्तमान राशि"),
    ("From natal Moon", "ஜன்ம சந்திரனிலிருந்து", "जन्म चंद्र से"),
    ("Star", "நட்சத்திரம்", "नक्षत्र"),
    ("house {0}", "{0}-ஆம் இடம்", "भाव {0}"),
    ("Saturn is in house {0} from the natal Moon: the 7½-year Sade Sati period is running.", "சனி ஜன்ம சந்திரனிலிருந்து {0}-ஆம் இடத்தில் உள்ளார்: ஏழரைச் சனி காலம் நடக்கிறது.", "शनि जन्म चंद्र से भाव {0} में है: साढ़ेसाती चल रही है।"),
    // ashtakavarga
    ("Bhinnashtakavarga (benefic points of each planet by sign)", "பின்னாஷ்டகவர்க்கம் (ஒவ்வொரு கிரகத்தின் சுப புள்ளிகள்)", "भिन्नाष्टकवर्ग (प्रत्येक ग्रह के शुभ बिंदु)"),
    ("Sarvashtakavarga (total of all planets)", "சர்வாஷ்டகவர்க்கம் (மொத்தம்)", "सर्वाष्टकवर्ग (कुल)"),
    ("Signs with 28 or more points are strong; under 25 are weak.", "28 அல்லது அதற்கு மேல் புள்ளிகள் உள்ள ராசிகள் வலுவானவை; 25-க்குக் குறைவானவை பலவீனமானவை.", "28 या अधिक बिंदु वाली राशियाँ बलवान, 25 से कम वाली दुर्बल हैं।"),
    // report tab
    ("A detailed report with charts, tables and readings opens in your web browser. To keep a PDF, use Print > Save as PDF there.", "சக்கரங்கள், அட்டவணைகள், பலன்களுடன் கூடிய விரிவான அறிக்கை உங்கள் இணைய உலாவியில் திறக்கும். PDF ஆகச் சேமிக்க அங்கு Print > Save as PDF பயன்படுத்தவும்.", "चार्ट, तालिकाओं और फलादेश के साथ विस्तृत रिपोर्ट आपके वेब ब्राउज़र में खुलेगी। PDF रखने के लिए वहाँ Print > Save as PDF चुनें।"),
    ("Create full report and open it", "முழு அறிக்கையை உருவாக்கித் திற", "पूर्ण रिपोर्ट बनाएँ और खोलें"),
    ("Save quick summary as text", "சுருக்கத்தை உரைக் கோப்பாகச் சேமி", "संक्षेप को टेक्स्ट फ़ाइल में सहेजें"),
    ("Copy summary", "சுருக்கத்தை நகலெடு", "संक्षेप कॉपी करें"),
    ("Report saved to {0}", "அறிக்கை {0} இல் சேமிக்கப்பட்டது", "रिपोर्ट {0} में सहेजी गई"),
    ("Could not save report: {0}", "அறிக்கையைச் சேமிக்க இயலவில்லை: {0}", "रिपोर्ट सहेजी नहीं जा सकी: {0}"),
    ("Report copied to the clipboard", "அறிக்கை நகலெடுக்கப்பட்டது", "रिपोर्ट कॉपी की गई"),
    ("Sections in the full report", "முழு அறிக்கையின் பகுதிகள்", "पूर्ण रिपोर्ट के भाग"),
    ("Quick summary", "சுருக்கம்", "संक्षेप"),
    // report sections
    ("Horoscope of {0}", "{0} அவர்களின் ஜாதகம்", "{0} की जन्म कुंडली"),
    ("Prepared with Vedic Astrology on {0}", "Vedic Astrology மூலம் {0} அன்று தயாரிக்கப்பட்டது", "Vedic Astrology द्वारा {0} को तैयार"),
    ("Contents", "உள்ளடக்கம்", "विषय सूची"),
    ("Birth details", "பிறப்பு விவரங்கள்", "जन्म विवरण"),
    ("Panchanga readings", "பஞ்சாங்கப் பலன்கள்", "पंचांग फल"),
    ("Planet positions", "கிரக நிலைகள்", "ग्रह स्थिति"),
    ("Charts", "சக்கரங்கள்", "कुंडलियाँ"),
    ("House table", "பாவ அட்டவணை", "भाव तालिका"),
    ("Planetary relationships", "கிரக உறவுகள்", "ग्रह संबंध"),
    ("Yogas and doshas", "யோகங்களும் தோஷங்களும்", "योग और दोष"),
    ("House readings", "பாவப் பலன்கள்", "भाव फल"),
    ("Dasha readings", "தசா பலன்கள்", "दशा फल"),
    ("Transit forecast", "கோசாரப் பலன்கள்", "गोचर फल"),
    ("Note", "குறிப்பு", "टिप्पणी"),
    ("Sex", "பாலினம்", "लिंग"),
    ("Date of birth", "பிறந்த தேதி", "जन्म तिथि"),
    ("Time of birth", "பிறந்த நேரம்", "जन्म समय"),
    ("Time zone", "நேர மண்டலம்", "समय क्षेत्र"),
    ("Place of birth", "பிறந்த இடம்", "जन्म स्थान"),
    ("Longitude & latitude", "தீர்க்க, அட்சரேகை", "देशांतर व अक्षांश"),
    ("Birth star – pada", "ஜன்ம நட்சத்திரம் – பாதம்", "जन्म नक्षत्र – चरण"),
    ("Birth rasi – lord", "ஜன்ம ராசி – அதிபதி", "जन्म राशि – स्वामी"),
    ("Lagna – lord", "லக்னம் – அதிபதி", "लग्न – स्वामी"),
    ("Tithi", "திதி", "तिथि"),
    ("Sunrise", "சூரிய உதயம்", "सूर्योदय"),
    ("Sunset", "சூரிய அஸ்தமனம்", "सूर्यास्त"),
    ("Day length (dinamana)", "பகல் அளவு (தினமானம்)", "दिनमान"),
    ("{0} h {1} min  ({2} nazhika)", "{0} ம {1} நி  ({2} நாழிகை)", "{0} घं {1} मि  ({2} घटी)"),
    ("Birth after sunrise", "உதயாதி நாழிகை", "इष्टकाल (सूर्योदय के बाद)"),
    ("{0} nazhika {1} vinazhika", "{0} நாழிகை {1} விநாழிகை", "{0} घटी {1} पल"),
    ("Local mean time correction", "உள்ளூர் சராசரி நேரத் திருத்தம்", "स्थानीय मध्यम समय संशोधन"),
    ("{0} min", "{0} நிமிடம்", "{0} मिनट"),
    ("Astrological weekday", "ஜோதிட வாரம்", "ज्योतिषीय वार"),
    ("Kali day count (Kalidina sankhya)", "கலி தின சங்கியை", "कलि दिन संख्या"),
    ("Dasa system", "தசா முறை", "दशा पद्धति"),
    ("Vimshottari, year = 365.25 days", "விம்சோத்தரி, ஆண்டு = 365.25 நாள்", "विंशोत्तरी, वर्ष = 365.25 दिन"),
    ("Gana, yoni, nadi", "கணம், யோனி, நாடி", "गण, योनि, नाड़ी"),
    ("Sun's rasi – star", "சூரிய ராசி – நட்சத்திரம்", "सूर्य राशि – नक्षत्र"),
    ("Western (tropical) Sun sign", "மேற்கத்திய சூரிய ராசி", "पाश्चात्य सूर्य राशि"),
    ("Yogi point – star – planet", "யோகி புள்ளி – நட்சத்திரம் – கிரகம்", "योगी बिंदु – नक्षत्र – ग्रह"),
    ("Avayogi star – planet", "அவயோகி நட்சத்திரம் – கிரகம்", "अवयोगी नक्षत्र – ग्रह"),
    ("Atma karaka – Karakamsa", "ஆத்ம காரகன் – காரகாம்சம்", "आत्मकारक – कारकांश"),
    ("Amatya karaka", "அமாத்ய காரகன்", "अमात्यकारक"),
    ("Lagna pada (Arudha)", "லக்ன ஆரூடம்", "लग्न आरूढ़"),
    ("Dhana pada (2nd Arudha)", "தன ஆரூடம்", "धन आरूढ़"),
    ("Ayanamsa", "அயனாம்சம்", "अयनांश"),
    ("Born {0}, {1}, in {2} star pada {3}, {4} {5}, {6} karana and {7} yoga, with the Moon in {8} rasi and {9} navamsa, in {10} lagna.", "{0}, {1} அன்று, {2} நட்சத்திரம் {3}-ஆம் பாதத்தில், {4} {5} திதியில், {6} கரணம், {7} யோகத்தில், சந்திரன் {8} ராசியிலும் {9} நவாம்சத்திலும் இருக்க, {10} லக்னத்தில் பிறப்பு.", "{0}, {1} को, {2} नक्षत्र के चरण {3} में, {4} {5} तिथि, {6} करण और {7} योग में, चंद्रमा {8} राशि और {9} नवांश में रहते हुए, {10} लग्न में जन्म।"),
    ("Weekday", "கிழமை", "वार"),
    ("Birth star", "ஜன்ம நட்சத்திரம்", "जन्म नक्षत्र"),
    ("Nithya yoga", "நித்ய யோகம்", "नित्य योग"),
    ("Lagna", "லக்னம்", "लग्न"),
    ("Nirayana (sidereal) longitudes", "நிராயன பாகைகள்", "निरयन भोगांश"),
    ("Sayana (tropical, western) longitudes", "சாயன (மேற்கத்திய) பாகைகள்", "सायन (पाश्चात्य) भोगांश"),
    ("Star lord / sub-lord / sub-sub-lord (KP)", "நட்சத்திர அதிபதி / உப அதிபதி / உப-உப அதிபதி (KP)", "नक्षत्र स्वामी / उप स्वामी / उप-उप स्वामी (KP)"),
    ("Sub-lord", "உப அதிபதி", "उप स्वामी"),
    ("Sub-sub-lord", "உப-உப அதிபதி", "उप-उप स्वामी"),
    ("Western sign", "மேற்கத்திய ராசி", "पाश्चात्य राशि"),
    ("Shodasavarga (divisional charts)", "சோடசவர்க்கம் (வர்க்க சக்கரங்கள்)", "षोडशवर्ग (वर्ग कुंडलियाँ)"),
    ("House lords", "பாவ அதிபதிகள்", "भाव स्वामी"),
    ("Lord placed in house", "அதிபதி இருக்கும் பாவம்", "स्वामी स्थित भाव"),
    ("Nature", "இயல்பு", "स्वभाव"),
    ("Benefic", "சுபர்", "शुभ"),
    ("Malefic", "பாபர்", "पाप"),
    ("Permanent (natural) friendship", "நிலையான (இயற்கை) நட்பு", "नैसर्गिक मैत्री"),
    ("Temporary friendship", "தற்காலிக நட்பு", "तात्कालिक मैत्री"),
    ("Five-fold friendship", "ஐந்து வகை நட்பு", "पंचधा मैत्री"),
    ("Friend", "நண்பர்", "मित्र"),
    ("Great friend", "அதி நண்பர்", "अधिमित्र"),
    ("Great enemy", "அதி பகைவர்", "अधिशत्रु"),
    ("Rows: the planet; columns: how it regards the other planet.", "வரிசை: கிரகம்; நெடுவரிசை: அது மற்ற கிரகத்தை எப்படிப் பார்க்கிறது.", "पंक्ति: ग्रह; स्तंभ: वह दूसरे ग्रह को कैसे मानता है।"),
    ("Balance of dasha at birth: {0} {1}", "பிறப்பின் போது தசை இருப்பு: {0} {1}", "जन्म के समय दशा शेष: {0} {1}"),
    ("Mahadashas", "மகா தசைகள்", "महादशाएँ"),
    ("{0} dasha – bhuktis", "{0} தசை – புக்திகள்", "{0} महादशा – अंतर्दशाएँ"),
    ("{0} dasha, {1} bhukti – antaras", "{0} தசை, {1} புக்தி – அந்தரங்கள்", "{0} महादशा, {1} अंतर्दशा – प्रत्यंतर"),
    ("Your {0} dasha runs from {1} to {2}.", "உங்கள் {0} தசை {1} முதல் {2} வரை.", "आपकी {0} महादशा {1} से {2} तक है।"),
    ("Bhuktis in this period", "இந்தத் தசையின் புக்திகள்", "इस दशा की अंतर्दशाएँ"),
    ("House {0}", "{0}-ஆம் பாவம்", "भाव {0}"),
    ("{0}: {1} in {2}, until {3}", "{0}: {1} இல் {2}, {3} வரை", "{0}: {1} में {2}, {3} तक"),
    ("Jupiter transits", "குரு பெயர்ச்சி", "गुरु गोचर"),
    ("Saturn transits", "சனிப் பெயர்ச்சி", "शनि गोचर"),
    ("Period", "காலம்", "अवधि"),
    ("Sign", "ராசி", "राशि"),
    ("From Moon", "சந்திரனிலிருந்து", "चंद्र से"),
    ("Reading", "பலன்", "फल"),
    // ---- web chrome (home, calendar, settings, nav) ----
    ("Report", "அறிக்கை", "रिपोर्ट"),
    ("Language", "மொழி", "भाषा"),
    ("Light / dark", "ஒளி / இருள்", "लाइट / डार्क"),
    ("New chart", "புதிய ஜாதகம்", "नई कुंडली"),
    ("Discover your Vedic birth chart", "உங்கள் வேத ஜாதகத்தை அறியுங்கள்", "अपनी वैदिक जन्म कुंडली जानें"),
    ("Enter your birth details and see your chart, planets, periods and a full reading — free.", "உங்கள் பிறப்பு விவரங்களை உள்ளிட்டு ஜாதகம், கிரகங்கள், தசைகள் மற்றும் முழு பலனைப் பாருங்கள் — இலவசம்.", "अपना जन्म विवरण भरें और अपनी कुंडली, ग्रह, दशाएँ और पूरा फलादेश देखें — निःशुल्क।"),
    ("Your birth details", "பிறப்பு விவரங்கள்", "जन्म विवरण"),
    ("Your reading", "உங்கள் பலன்", "आपका फलादेश"),
    ("Your name", "உங்கள் பெயர்", "आपका नाम"),
    ("Date of birth", "பிறந்த தேதி", "जन्म तिथि"),
    ("Time of birth", "பிறந்த நேரம்", "जन्म समय"),
    ("Birth place", "பிறந்த இடம்", "जन्म स्थान"),
    ("City, State, Country", "நகரம், மாநிலம், நாடு", "शहर, राज्य, देश"),
    ("Coordinates & time zone (filled by Fetch)", "ஆயத்தொலைவுகள் & நேர மண்டலம் (Fetch நிரப்பும்)", "निर्देशांक और समय क्षेत्र (Fetch से भरे)"),
    ("Latitude (N+)", "அட்சரேகை (வ+)", "अक्षांश (उ+)"),
    ("Longitude (E+)", "தீர்க்கரேகை (கி+)", "देशांतर (पू+)"),
    ("Time zone (hours)", "நேர மண்டலம் (மணி)", "समय क्षेत्र (घंटे)"),
    ("Zone", "மண்டலம்", "क्षेत्र"),
    ("Chart style", "அட்டவணை பாணி", "कुंडली शैली"),
    ("Advanced calculation options", "மேம்பட்ட கணக்கீட்டு விருப்பங்கள்", "उन्नत गणना विकल्प"),
    ("Rahu / Ketu", "ராகு / கேது", "राहु / केतु"),
    ("Mean node", "சராசரி நோடு", "मध्य नोड"),
    ("True node", "உண்மை நோடு", "वास्तविक नोड"),
    ("Calculation engine", "கணக்கீட்டு இயந்திரம்", "गणना इंजन"),
    ("Built-in formulas", "உள்ளமைந்த சூத்திரங்கள்", "अंतर्निहित सूत्र"),
    ("Add an AI-written reading", "AI எழுதிய பலனைச் சேர்", "AI-लिखित फलादेश जोड़ें"),
    ("See my chart", "என் ஜாதகத்தைக் காட்டு", "मेरी कुंडली देखें"),
    ("Calendar for this place", "இந்த இடத்திற்கான நாட்காட்டி", "इस स्थान का कैलेंडर"),
    ("Save details", "விவரங்களைச் சேமி", "विवरण सहेजें"),
    ("Load details", "விவரங்களை ஏற்று", "विवरण लोड करें"),
    ("Add an AI-written reading to each section.", "ஒவ்வொரு பிரிவிலும் AI எழுதிய பலனைச் சேர்க்கிறது.", "प्रत्येक अनुभाग में AI-लिखित फलादेश जोड़ता है।"),
    ("Ask about {0}'s chart", "{0} இன் ஜாதகம் பற்றி கேளுங்கள்", "{0} की कुंडली के बारे में पूछें"),
    ("Ask", "கேள்", "पूछें"),
    ("e.g. When is a good period for my career?", "எ.கா. என் தொழிலுக்கு நல்ல காலம் எப்போது?", "उदा. मेरे करियर के लिए अच्छा समय कब है?"),
    ("The AI answers from this chart and the knowledge base only.", "AI இந்த ஜாதகம் மற்றும் அறிவுத் தளத்திலிருந்து மட்டுமே பதிலளிக்கும்.", "AI केवल इस कुंडली और ज्ञान-आधार से उत्तर देता है।"),
    ("Turn on the AI writer in Settings to ask questions.", "கேள்விகள் கேட்க அமைப்புகளில் AI எழுத்தாளரை இயக்கவும்.", "प्रश्न पूछने हेतु सेटिंग्स में AI लेखक चालू करें।"),
    ("Report language", "அறிக்கை மொழி", "रिपोर्ट भाषा"),
    ("Go to month", "மாதத்திற்குச் செல்", "महीने पर जाएँ"),
    ("Sign (rashi) names", "ராசி பெயர்கள்", "राशि के नाम"),
    ("Regional / Vedic", "பிராந்திய / வேத", "क्षेत्रीय / वैदिक"),
    ("Western (Aries–Pisces)", "மேற்கத்திய (Aries–Pisces)", "पश्चिमी (Aries–Pisces)"),
    ("Both — Kumbham (Aquarius)", "இரண்டும் — கும்பம் (Aquarius)", "दोनों — कुम्भ (Aquarius)"),
    ("Knowledge base", "அறிவுத் தளம்", "ज्ञान-आधार"),
    ("When the AI writer is on, it answers using only this chart plus the passages it finds here (retrieval-augmented, RAG). This is what will be consulted:", "AI எழுத்தாளர் இயக்கத்தில் இருக்கும்போது, இந்த ஜாதகம் மற்றும் இங்கு கிடைக்கும் பகுதிகளை மட்டுமே பயன்படுத்திப் பதிலளிக்கும் (RAG). இங்கே பயன்படுத்தப்படுபவை:", "जब AI लेखक चालू होता है, तो यह केवल इस कुंडली और यहाँ मिलने वाले अंशों से उत्तर देता है (RAG)। यही परामर्श किया जाएगा:"),
    ("rule records", "விதி பதிவுகள்", "नियम रिकॉर्ड"),
    ("glossary entries", "சொற்களஞ்சிய உள்ளீடுகள்", "शब्दावली प्रविष्टियाँ"),
    ("searchable passages", "தேடக்கூடிய பகுதிகள்", "खोज योग्य अंश"),
    ("Larger text", "பெரிய எழுத்து", "बड़ा टेक्स्ट"),
    ("Smaller text", "சிறிய எழுத்து", "छोटा टेक्स्ट"),
    ("Turn on the AI writer in Settings first.", "AI எழுத்தாளரை முதலில் அமைப்புகளில் இயக்கவும்.", "पहले सेटिंग्स में AI लेखक चालू करें।"),
    // settings
    ("Set up the optional AI writer and choose how the site looks. The API key and base URL live on the server and are never shown or sent to your browser.", "விருப்பமான AI எழுத்தாளரை அமைத்து, தளத்தின் தோற்றத்தைத் தேர்வுசெய்யுங்கள். API கீ மற்றும் அடிப்படை URL சேவையகத்தில் இருக்கும், உலாவிக்கு ஒருபோதும் காட்டப்படாது.", "वैकल्पिक AI लेखक सेट करें और चुनें कि साइट कैसी दिखे। API कुंजी और बेस URL सर्वर पर रहते हैं और कभी ब्राउज़र को नहीं दिखाए या भेजे जाते।"),
    ("AI writer", "AI எழுத்தாளர்", "AI लेखक"),
    ("Model", "மாடல்", "मॉडल"),
    ("Load models", "மாடல்களை ஏற்று", "मॉडल लोड करें"),
    ("Test connection", "இணைப்பைச் சோதி", "कनेक्शन जांचें"),
    ("Creativity (temperature 0–1)", "படைப்பாற்றல் (temperature 0–1)", "रचनात्मकता (temperature 0–1)"),
    ("Knowledge passages per section", "பிரிவுக்கான அறிவுத் தொகுதிகள்", "प्रति अनुभाग ज्ञान अंश"),
    ("Default calculation engine", "இயல்பு கணக்கீட்டு இயந்திரம்", "डिफ़ॉल्ट गणना इंजन"),
    ("AI instructions", "AI வழிமுறைகள்", "AI निर्देश"),
    ("These guide every AI reading and answer. Leave blank to use the built-in instructions.", "இவை ஒவ்வொரு AI பலனையும் பதிலையும் வழிநடத்தும். உள்ளமைந்த வழிமுறைகளைப் பயன்படுத்த வெறுமையாக விடவும்.", "ये हर AI फलादेश और उत्तर का मार्गदर्शन करते हैं। अंतर्निहित निर्देश उपयोग करने हेतु खाली छोड़ें।"),
    ("Save settings", "அமைப்புகளைச் சேமி", "सेटिंग्स सहेजें"),
    ("Appearance", "தோற்றம்", "रूप"),
    ("Theme — remembered on this device.", "தீம் — இந்தச் சாதனத்தில் நினைவில் வைக்கப்படும்.", "थीम — इस डिवाइस पर याद रखी जाती है।"),
    ("Match device", "சாதனத்தைப் பின்பற்று", "डिवाइस के अनुसार"),
    // calendar headers / nav
    ("Prev", "முந்தைய", "पिछला"),
    ("Next", "அடுத்த", "अगला"),
    ("Tamil / Vedic calendar", "தமிழ் / வேத நாட்காட்டி", "तमिल / वैदिक पंचांग"),
    ("Date", "தேதி", "तारीख़"),
    ("Tamil date", "தமிழ் தேதி", "तमिल तिथि"),
    ("Lunar day (tithi)", "திதி", "तिथि"),
    ("Star (nakshatra)", "நட்சத்திரம்", "नक्षत्र"),
    ("Observance", "விரதம் / விழா", "पर्व"),
    ("Sunrise / sunset", "சூரிய உதயம் / அஸ்தமனம்", "सूर्योदय / सूर्यास्त"),
    ("Yamagandam (avoid)", "யமகண்டம் (தவிர்க்கவும்)", "यमगंड (टालें)"),
    ("Gulika kalam (avoid new starts)", "குளிகை காலம் (புதிய தொடக்கங்களைத் தவிர்)", "गुलिक काल (नई शुरुआत टालें)"),
    ("Abhijit muhurta (auspicious)", "அபிஜித் முகூர்த்தம் (நல்லது)", "अभिजित मुहूर्त (शुभ)"),
    ("Brahma muhurta (auspicious)", "பிரம்ம முகூர்த்தம் (நல்லது)", "ब्रह्म मुहूर्त (शुभ)"),
    ("Each day shows the Tamil date, and the lunar day (tithi) and star (nakshatra) at sunrise. Pick a day for its timings.", "ஒவ்வொரு நாளும் தமிழ் தேதியையும், சூரிய உதயத்தில் திதி மற்றும் நட்சத்திரத்தையும் காட்டுகிறது. நேரங்களுக்கு ஒரு நாளைத் தேர்ந்தெடுக்கவும்.", "हर दिन तमिल तिथि, और सूर्योदय के समय की तिथि व नक्षत्र दिखाता है। समय देखने हेतु कोई दिन चुनें।"),
    ("{0} year", "{0} ஆண்டு", "{0} वर्ष"),
    // weekday reckoning
    ("before sunrise", "சூரிய உதயத்திற்கு முன்", "सूर्योदय से पहले"),
    ("Astrological weekday (vara, from sunrise)", "ஜோதிட வாரம் (வாரம், சூரிய உதயத்திலிருந்து)", "ज्योतिषीय वार (सूर्योदय से)"),
    ("Civil weekday (calendar)", "நாட்காட்டி வாரம்", "कैलेंडर वार"),
    ("Weekday & dates", "வாரம் & தேதிகள்", "वार और तिथियाँ"),
    ("Astrological (sunrise to sunrise)", "ஜோதிடம் (சூரிய உதயம் முதல்)", "ज्योतिषीय (सूर्योदय से सूर्योदय)"),
    ("Civil (midnight to midnight)", "சிவில் (நள்ளிரவு முதல்)", "सिविल (मध्यरात्रि से)"),
    ("The weekday the report shows. Vedic astrology counts the day from sunrise, so a pre-dawn birth shows the previous weekday.", "அறிக்கையில் காட்டப்படும் வாரம். வேத ஜோதிடம் நாளைச் சூரிய உதயத்திலிருந்து கணக்கிடுகிறது, எனவே விடியலுக்கு முந்தைய பிறப்பு முந்தைய வாரத்தைக் காட்டும்.", "रिपोर्ट में दिखने वाला वार। वैदिक ज्योतिष दिन की गणना सूर्योदय से करता है, इसलिए भोर से पहले का जन्म पिछला वार दिखाता है।"),
    ("This report is calculated from the birth details given, using standard Vedic astrology rules. Readings describe tendencies, not certainties; please use them as general guidance.", "இந்த அறிக்கை கொடுக்கப்பட்ட பிறப்பு விவரங்களிலிருந்து வழக்கமான வேத ஜோதிட விதிகளின்படி கணக்கிடப்பட்டது. பலன்கள் போக்குகளைக் குறிக்கின்றன, உறுதியானவை அல்ல; பொது வழிகாட்டுதலாகப் பயன்படுத்தவும்.", "यह रिपोर्ट दिए गए जन्म विवरण से मानक वैदिक ज्योतिष नियमों के अनुसार बनाई गई है। फलादेश प्रवृत्तियाँ बताते हैं, निश्चितता नहीं; कृपया इन्हें सामान्य मार्गदर्शन मानें।"),
    ("Planets in each house according to the house cusps (Sripati).", "பாவ சந்திகளின்படி ஒவ்வொரு பாவத்திலும் உள்ள கிரகங்கள் (ஸ்ரீபதி).", "भाव संधियों के अनुसार प्रत्येक भाव में ग्रह (श्रीपति)।"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn festival_names_and_islamic_label_are_translated() {
        // The tradition label is "Islamic" (not the adherent noun "Muslim").
        assert!(UI.iter().any(|&(e, _, _)| e == "Islamic"), "Islamic tradition label missing");
        assert!(!UI.iter().any(|&(e, _, _)| e == "Muslim"), "label should be Islamic, not Muslim");
        for name in ["Diwali", "Holi", "Makar Sankranti", "Vesak", "Mahavir Jayanti", "Christmas", "Good Friday", "Eid al-Fitr", "Ramadan"] {
            let (_, ta, hi) = *UI.iter().find(|&&(e, _, _)| e == name).unwrap_or_else(|| panic!("UI missing {name}"));
            assert!(!ta.is_empty() && ta != name, "{name} has no Tamil");
            assert!(!hi.is_empty() && hi != name, "{name} has no Hindi");
        }
    }
}
