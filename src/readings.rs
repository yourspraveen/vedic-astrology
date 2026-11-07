//! Detailed reading texts for the full report (house titles, bhukti readings, yoga results,
//! benefic/malefic explanations, remedies), in English, Tamil and Hindi. Written for this app
//! from traditional meanings; no other program's wording is used. Planet-in-house readings live
//! in pih_en / pih_ta / pih_hi.

use crate::i18n::li;

/// Reading for planet `g` (Sun = 0 … Ketu = 8) in house `h` (1..12).
pub fn planet_in_house(g: usize, h: usize) -> &'static str {
    let table = match li() {
        1 => &crate::pih_ta::PIH,
        2 => &crate::pih_hi::PIH,
        _ => &crate::pih_en::PIH,
    };
    table[g % 9][(h + 11) % 12]
}
pub fn house_title(h: usize) -> &'static str {
    HOUSE_TITLE[li()][(h + 11) % 12]
}
/// Topics a bhukti lord brings forward.
pub fn bhukti_topics(g: usize) -> &'static str {
    BHUKTI[li()][g % 9]
}
pub fn t(k: R) -> &'static str {
    TPL[k as usize][li()]
}

#[derive(Clone, Copy)]
pub enum R {
    LordLink = 0,
    JupAspect,
    SatAspect,
    MarsAspect,
    Drekkana1,
    Drekkana2,
    Drekkana3,
    BhuktiIntro,
    RelFriend,
    RelNeutral,
    RelEnemy,
    BhuktiPlace,
    DashaStrong,
    DashaWeak,
    DashaAverage,
    FuncNature,
    NaturalIntro,
    PakshaRule,
    MoonWith,
    MoonWithout,
    MercuryRule,
    MercuryBad,
    MercuryGood,
    LordshipRules,
    StarIntro,
    StarAdverse,
    StarAdvice,
    DashaRemedyIntro,
    DashaRemedyFor,
    NoDashaRemedy,
    KujaRemedy,
    RahuRemedy,
    KetuRemedy,
    SummaryIntro,
    Benefic,
    Malefic,
    Neutral,
    Logic,
}

/// Classical result of a yoga, by its English name.
pub fn yoga_result(name: &str) -> &'static str {
    YOGA_RESULTS.iter().find(|y| y.0 == name).map(|y| y.1[li()]).unwrap_or("")
}

pub struct Remedy {
    pub dress: &'static str,
    pub fasting: &'static str,
    pub alms: &'static str,
    pub flowers: &'static str,
    pub gem: &'static str,
    pub worship: &'static str,
    pub mantra: &'static str,
}

/// Traditional remedies for a planet (Sun = 0 … Ketu = 8).
pub fn remedy(g: usize) -> Remedy {
    let r = &REMEDY[li()][g % 9];
    Remedy { dress: r[0], fasting: r[1], alms: r[2], flowers: r[3], gem: r[4], worship: r[5], mantra: r[6] }
}

pub fn remedy_labels() -> [&'static str; 7] {
    REMEDY_LABELS[li()]
}

const REMEDY_LABELS: [[&str; 7]; 3] = [
    ["Dress", "Fasting (vratham)", "Alms (daanam)", "Flowers", "Gemstone", "Worship", "Mantra"],
    ["உடை", "விரதம்", "தானம்", "மலர்கள்", "ரத்தினம்", "வழிபாடு", "மந்திரம்"],
    ["वस्त्र", "व्रत", "दान", "पुष्प", "रत्न", "उपासना", "मंत्र"],
];

const TPL: [[&str; 3]; 38] = [
    ["So the matters of {0} become linked with {1}.", "ஆகவே {0} தொடர்பான விஷயங்கள் {1} உடன் இணைகின்றன.", "इस प्रकार {0} से जुड़े विषय {1} से जुड़ जाते हैं।"],
    ["Jupiter aspects this house, which protects and expands its matters.", "குரு இந்தப் பாவத்தைப் பார்ப்பதால் இதன் விஷயங்கள் பாதுகாக்கப்பட்டு வளரும்.", "गुरु की इस भाव पर दृष्टि है, जो इसके विषयों की रक्षा और वृद्धि करती है।"],
    ["Saturn aspects this house: results come with delay, but steady effort makes them lasting.", "சனி இந்தப் பாவத்தைப் பார்ப்பதால் பலன்கள் தாமதமாக வரும்; ஆனால் விடாமுயற்சி அவற்றை நிலைக்கச் செய்யும்.", "शनि की इस भाव पर दृष्टि है: फल देर से मिलते हैं, पर निरंतर प्रयास उन्हें स्थायी बनाता है।"],
    ["Mars aspects this house, adding energy and drive, along with occasional haste or conflict.", "செவ்வாய் இந்தப் பாவத்தைப் பார்ப்பதால் ஆற்றலும் உந்துதலும் கூடும்; சில நேரம் அவசரமும் மோதலும் ஏற்படலாம்.", "मंगल की इस भाव पर दृष्टि है, जिससे ऊर्जा और जोश बढ़ता है, कभी-कभी जल्दबाज़ी या विवाद भी।"],
    [
        "Your Lagna lies in the first drekkana (0°–10°) of {0}, so your nature strongly reflects the qualities of {0} itself.",
        "உங்கள் லக்னம் {0} ராசியின் முதல் திரேக்காணத்தில் (0°–10°) உள்ளது; ஆகவே உங்கள் இயல்பில் {0} ராசியின் குணங்கள் வலுவாகத் தெரியும்.",
        "आपका लग्न {0} के पहले द्रेष्काण (0°–10°) में है, इसलिए आपके स्वभाव में {0} के गुण प्रबल रूप से झलकते हैं।",
    ],
    [
        "Your Lagna lies in the middle drekkana (10°–20°), coloured by {0}: creativity, intelligence and love of children add to your character.",
        "உங்கள் லக்னம் நடு திரேக்காணத்தில் (10°–20°), {0} ராசியின் தாக்கத்துடன் உள்ளது: படைப்பாற்றல், அறிவு, குழந்தைகள் மீதான அன்பு உங்கள் குணத்தில் சேரும்.",
        "आपका लग्न मध्य द्रेष्काण (10°–20°) में है, जिस पर {0} का प्रभाव है: रचनात्मकता, बुद्धि और संतान-प्रेम आपके चरित्र में जुड़ते हैं।",
    ],
    [
        "Your Lagna lies in the last drekkana (20°–30°), coloured by {0}: principles, faith and the search for meaning shape your character.",
        "உங்கள் லக்னம் கடைசி திரேக்காணத்தில் (20°–30°), {0} ராசியின் தாக்கத்துடன் உள்ளது: கொள்கைகள், நம்பிக்கை, வாழ்வின் பொருளைத் தேடுதல் உங்கள் குணத்தை வடிவமைக்கும்.",
        "आपका लग्न अंतिम द्रेष्काण (20°–30°) में है, जिस पर {0} का प्रभाव है: सिद्धांत, आस्था और जीवन के अर्थ की खोज आपके चरित्र को गढ़ते हैं।",
    ],
    ["The sub-period of {0} in the {1} dasa brings {2}.", "{1} தசையில் {0} புக்தி {2} ஆகியவற்றைக் கொண்டுவரும்.", "{1} महादशा में {0} की अंतर्दशा {2} लाती है।"],
    ["{0} and {1} are friends, so this sub-period generally runs smoothly.", "{0}, {1} இருவரும் நண்பர்கள்; ஆகவே இந்தப் புக்தி பொதுவாகச் சீராக நடக்கும்.", "{0} और {1} मित्र हैं, इसलिए यह अंतर्दशा सामान्यतः सुचारु रहती है।"],
    ["{0} and {1} are neutral to each other, so results are moderate.", "{0}, {1} இருவரும் சம உறவில் உள்ளனர்; பலன்கள் மிதமாக இருக்கும்.", "{0} और {1} परस्पर सम हैं, इसलिए फल मध्यम रहते हैं।"],
    ["{0} and {1} are not friendly, so expect some friction; act with patience.", "{0}, {1} இருவரும் நட்பற்றவர்கள்; சில முரண்பாடுகள் வரலாம், பொறுமையுடன் செயல்படவும்.", "{0} और {1} मित्र नहीं हैं, इसलिए कुछ टकराव संभव है; धैर्य से काम लें।"],
    ["{0} sits in house {1} of your chart, so matters of {2} become active.", "{0} உங்கள் ஜாதகத்தில் {1}-ஆம் பாவத்தில் உள்ளதால் {2} தொடர்பான விஷயங்கள் செயல்படும்.", "{0} आपकी कुंडली के भाव {1} में है, इसलिए {2} से जुड़े विषय सक्रिय होते हैं।"],
    ["{0} is strong in your horoscope ({1}), so this dasa will generally be good for you.", "{0} உங்கள் ஜாதகத்தில் வலுவாக ({1}) உள்ளதால் இந்தத் தசை பொதுவாக நன்மை தரும்.", "आपकी कुंडली में {0} बलवान ({1}) है, इसलिए यह दशा सामान्यतः शुभ रहेगी।"],
    ["{0} is weak in your horoscope ({1}), so results come with effort; the remedies section helps.", "{0} உங்கள் ஜாதகத்தில் பலவீனமாக ({1}) உள்ளதால் முயற்சியால் பலன் கிடைக்கும்; பரிகாரங்கள் உதவும்.", "आपकी कुंडली में {0} दुर्बल ({1}) है, इसलिए फल प्रयास से मिलेंगे; उपाय सहायक होंगे।"],
    ["{0} has average strength in your horoscope ({1}); results depend largely on the sub-period lords.", "{0} உங்கள் ஜாதகத்தில் சராசரி பலத்துடன் ({1}) உள்ளார்; பலன்கள் பெரும்பாலும் புக்தி நாதர்களைப் பொறுத்தது.", "आपकी कुंडली में {0} मध्यम बल ({1}) का है; फल मुख्यतः अंतर्दशा स्वामियों पर निर्भर करेंगे।"],
    ["For your lagna, {0} is a functional {1} (lord of house {2}).", "உங்கள் லக்னத்திற்கு {0} {1} ஆவார் ({2}-ஆம் பாவ அதிபதி).", "आपके लग्न के लिए {0} {1} है (भाव {2} का स्वामी)।"],
    [
        "Jupiter, Venus and a Moon with paksha bala are natural benefics; the Sun, Mars, Saturn, Rahu and Ketu are natural malefics. Mercury takes on the nature of the planets it is with.",
        "குரு, சுக்கிரன், பக்ஷ பலமுள்ள சந்திரன் இயற்கைச் சுபர்கள்; சூரியன், செவ்வாய், சனி, ராகு, கேது இயற்கைப் பாபர்கள். புதன் தன்னுடன் சேர்ந்துள்ள கிரகங்களின் இயல்பைப் பெறுவார்.",
        "गुरु, शुक्र और पक्षबल युक्त चंद्रमा नैसर्गिक शुभ ग्रह हैं; सूर्य, मंगल, शनि, राहु और केतु नैसर्गिक पाप ग्रह हैं। बुध जिन ग्रहों के साथ होता है, उन्हीं का स्वभाव ले लेता है।",
    ],
    [
        "The Moon has paksha bala from Shashthi of the bright half up to Shashthi of the dark half.",
        "வளர்பிறை சஷ்டி முதல் தேய்பிறை சஷ்டி வரை சந்திரனுக்கு பக்ஷ பலம் உண்டு.",
        "शुक्ल पक्ष की षष्ठी से कृष्ण पक्ष की षष्ठी तक चंद्रमा को पक्षबल प्राप्त होता है।",
    ],
    ["In your horoscope the Moon has paksha bala and is benefic.", "உங்கள் ஜாதகத்தில் சந்திரனுக்கு பக்ஷ பலம் உண்டு; சந்திரன் சுபர்.", "आपकी कुंडली में चंद्रमा पक्षबल युक्त और शुभ है।"],
    ["In your horoscope the Moon is without paksha bala and is malefic.", "உங்கள் ஜாதகத்தில் சந்திரனுக்கு பக்ஷ பலம் இல்லை; சந்திரன் பாபர்.", "आपकी कुंडली में चंद्रमा पक्षबल रहित और पाप है।"],
    ["Mercury turns malefic when it is with malefic planets.", "பாப கிரகங்களுடன் சேர்ந்தால் புதன் பாபராக மாறுவார்.", "पाप ग्रहों के साथ होने पर बुध पाप बन जाता है।"],
    ["In your chart Mercury is with a malefic and so acts as a malefic.", "உங்கள் ஜாதகத்தில் புதன் பாப கிரகத்துடன் சேர்ந்திருப்பதால் பாபராகச் செயல்படுவார்.", "आपकी कुंडली में बुध पाप ग्रह के साथ है, इसलिए पाप फल देता है।"],
    ["In your chart Mercury is free of malefic company and acts as a benefic.", "உங்கள் ஜாதகத்தில் புதன் பாப கிரகச் சேர்க்கையின்றி சுபராகச் செயல்படுவார்.", "आपकी कुंडली में बुध पाप ग्रहों के साथ नहीं है, इसलिए शुभ फल देता है।"],
    [
        "Beyond their natural character, planets act through the houses they rule. Lords of the 1st, 5th and 9th are always benefic. Lords of the 3rd, 6th and 11th lean malefic. A natural malefic that rules a kendra (4th, 7th, 10th) improves, while a natural benefic that rules a kendra loses some goodness (kendradhipathya dosha). Lords of the 2nd and 12th are neutral, and the 8th lord is judged mainly by its other house.",
        "இயற்கை இயல்பைத் தாண்டி, கிரகங்கள் தாங்கள் ஆளும் பாவங்கள் மூலம் செயல்படுகின்றன. 1, 5, 9 ஆம் பாவ அதிபதிகள் எப்போதும் சுபர். 3, 6, 11 ஆம் அதிபதிகள் பாபத்தன்மை உடையவர். கேந்திரத்தை (4, 7, 10) ஆளும் இயற்கைப் பாபர் மேம்படுவார்; கேந்திரத்தை ஆளும் இயற்கைச் சுபர் நன்மை குறைவார் (கேந்திராதிபத்ய தோஷம்). 2, 12 ஆம் அதிபதிகள் சமம்; 8 ஆம் அதிபதி அவர் ஆளும் மற்றொரு பாவத்தால் மதிப்பிடப்படுவார்.",
        "स्वाभाविक प्रकृति के अतिरिक्त, ग्रह अपने स्वामित्व वाले भावों के माध्यम से फल देते हैं। 1, 5 और 9 भाव के स्वामी सदा शुभ होते हैं। 3, 6 और 11 के स्वामी पाप प्रवृत्ति के होते हैं। केंद्र (4, 7, 10) का स्वामी पाप ग्रह शुभ बनता है, जबकि केंद्र का स्वामी शुभ ग्रह अपनी शुभता खोता है (केंद्राधिपत्य दोष)। 2 और 12 के स्वामी सम होते हैं, और अष्टमेश का निर्णय उसके दूसरे भाव से होता है।",
    ],
    ["You were born in {0} star, ruled by {1}; your birth sign {2} is ruled by {3}.", "நீங்கள் {0} நட்சத்திரத்தில் பிறந்தீர்கள், அதன் அதிபதி {1}; உங்கள் ஜன்ம ராசி {2}, அதன் அதிபதி {3}.", "आपका जन्म {0} नक्षत्र में हुआ, जिसका स्वामी {1} है; आपकी जन्म राशि {2} का स्वामी {3} है।"],
    [
        "Counting from your birth star, the 3rd, 5th and 7th stars (Vipat, Pratyak and Naidhana taras) are ruled by {0}, so their dasa periods may bring more challenges.",
        "உங்கள் ஜன்ம நட்சத்திரத்திலிருந்து 3, 5, 7 ஆம் நட்சத்திரங்கள் (விபத், பிரத்யக், நைதன தாரைகள்) {0} ஆளுகையில் உள்ளன; ஆகவே அவர்களின் தசைகள் கூடுதல் சவால்களைத் தரலாம்.",
        "आपके जन्म नक्षत्र से तीसरे, पाँचवें और सातवें नक्षत्र (विपत, प्रत्यक् और नैधन तारा) के स्वामी {0} हैं, इसलिए उनकी दशाएँ अधिक चुनौतीपूर्ण हो सकती हैं।",
    ],
    [
        "During such periods, speak and act with restraint, avoid needless disputes and begin important work on favourable days. Regular prayer to {0}, the lord of your star, and kindness to animals, birds and plants are traditionally recommended.",
        "அத்தகைய காலங்களில் சொல்லிலும் செயலிலும் கட்டுப்பாடு காக்கவும், தேவையற்ற வாக்குவாதங்களைத் தவிர்க்கவும், முக்கிய செயல்களை நல்ல நாட்களில் தொடங்கவும். உங்கள் நட்சத்திர அதிபதியான {0}-ஐ வழிபடுவதும், விலங்குகள், பறவைகள், செடிகளிடம் கருணை காட்டுவதும் பாரம்பரியமாகப் பரிந்துரைக்கப்படுகிறது.",
        "ऐसे समय में वाणी और कर्म में संयम रखें, अनावश्यक विवाद से बचें और महत्वपूर्ण कार्य शुभ दिनों में आरंभ करें। अपने नक्षत्र स्वामी {0} की नियमित उपासना तथा पशु-पक्षियों और पौधों के प्रति दया पारंपरिक रूप से लाभकारी मानी जाती है।",
    ],
    [
        "The dasas below are less favourable in this horoscope because their lord is a functional malefic, weak by sign, placed in the 6th, 8th or 12th house, or rules an adverse star. Traditional remedies for each are listed.",
        "கீழ்க்கண்ட தசைகள் இந்த ஜாதகத்தில் குறைவான சாதகமானவை: அவற்றின் அதிபதி பாபராக, ராசியில் பலவீனமாக, 6, 8, 12 இல் அல்லது பாதகமான நட்சத்திர அதிபதியாக உள்ளார். ஒவ்வொன்றுக்குமான பாரம்பரிய பரிகாரங்கள் தரப்பட்டுள்ளன.",
        "नीचे दी गई दशाएँ इस कुंडली में कम अनुकूल हैं, क्योंकि उनका स्वामी पाप फलदायी, राशि में दुर्बल, 6, 8 या 12 भाव में स्थित, या प्रतिकूल नक्षत्र का स्वामी है। प्रत्येक के पारंपरिक उपाय दिए गए हैं।",
    ],
    ["{0} dasa ({1} – {2}): {3}", "{0} தசை ({1} – {2}): {3}", "{0} महादशा ({1} – {2}): {3}"],
    ["None of the coming dasas is clearly unfavourable; regular prayer and good conduct are enough.", "வரவிருக்கும் தசைகளில் எதுவும் தெளிவாகப் பாதகமில்லை; வழக்கமான வழிபாடும் நல்லொழுக்கமும் போதும்.", "आने वाली कोई भी दशा स्पष्ट रूप से प्रतिकूल नहीं है; नियमित उपासना और सदाचार पर्याप्त हैं।"],
    [
        "For Kuja dosha: fast on Tuesdays, worship Lord Murugan or Hanuman, and donate red lentils and red cloth.",
        "செவ்வாய் தோஷத்திற்கு: செவ்வாய்க்கிழமை விரதம் இருக்கவும், முருகப்பெருமான் அல்லது அனுமனை வழிபடவும், துவரம் பருப்பு, சிவப்பு ஆடை தானம் செய்யவும்.",
        "मंगल दोष के लिए: मंगलवार का व्रत रखें, कार्तिकेय या हनुमान जी की उपासना करें, और मसूर दाल व लाल वस्त्र दान करें।",
    ],
    [
        "For Rahu dosha: worship Goddess Durga, light a lamp on Saturdays and donate black gram and a blanket.",
        "ராகு தோஷத்திற்கு: துர்கை அம்மனை வழிபடவும், சனிக்கிழமைகளில் விளக்கேற்றவும், உளுந்து, போர்வை தானம் செய்யவும்.",
        "राहु दोष के लिए: माँ दुर्गा की उपासना करें, शनिवार को दीपक जलाएँ, और उड़द व कंबल दान करें।",
    ],
    [
        "For Ketu dosha: worship Lord Ganesha, especially on Tuesdays, and donate horse gram and multicoloured cloth.",
        "கேது தோஷத்திற்கு: விநாயகப் பெருமானை, குறிப்பாகச் செவ்வாய்க்கிழமைகளில் வழிபடவும், கொள்ளு, பல வண்ண ஆடை தானம் செய்யவும்.",
        "केतु दोष के लिए: विशेषकर मंगलवार को गणेश जी की उपासना करें, और कुलथी व बहुरंगी वस्त्र दान करें।",
    ],
    ["The main points of this horoscope at a glance.", "இந்த ஜாதகத்தின் முக்கிய அம்சங்கள் ஒரே பார்வையில்.", "इस कुंडली के मुख्य बिंदु एक नज़र में।"],
    ["benefic", "சுபர்", "शुभ"],
    ["malefic", "பாபர்", "पाप"],
    ["neutral", "சமர்", "सम"],
    ["Logic", "விதி", "नियम"],
];

const HOUSE_TITLE: [[&str; 12]; 3] = [
    [
        "Personality, physical build and status",
        "Wealth, family, land and possessions",
        "Siblings, courage and communication",
        "Property, education, comforts and mother",
        "Children, mind and intelligence",
        "Diseases, enemies and obstacles",
        "Marriage and partnership",
        "Longevity and difficulties",
        "Fortune, prosperity and inheritance",
        "Profession and career",
        "Income and gains",
        "Expenditure and losses",
    ],
    [
        "ஆளுமை, உடலமைப்பு, அந்தஸ்து",
        "செல்வம், குடும்பம், நிலம், உடைமைகள்",
        "சகோதரர்கள், தைரியம், தொடர்பு",
        "சொத்து, கல்வி, சுகம், தாய்",
        "குழந்தைகள், மனம், அறிவு",
        "நோய்கள், எதிரிகள், தடைகள்",
        "திருமணம், கூட்டாண்மை",
        "ஆயுள், சிரமங்கள்",
        "பாக்கியம், செழிப்பு, பரம்பரைச் சொத்து",
        "தொழில், உத்தியோகம்",
        "வருமானம், லாபம்",
        "செலவுகள், இழப்புகள்",
    ],
    [
        "व्यक्तित्व, शारीरिक बनावट और प्रतिष्ठा",
        "धन, परिवार, भूमि और संपत्ति",
        "भाई-बहन, साहस और संवाद",
        "संपत्ति, शिक्षा, सुख और माता",
        "संतान, मन और बुद्धि",
        "रोग, शत्रु और बाधाएँ",
        "विवाह और साझेदारी",
        "आयु और कठिनाइयाँ",
        "भाग्य, समृद्धि और विरासत",
        "व्यवसाय और करियर",
        "आय और लाभ",
        "व्यय और हानि",
    ],
];

const BHUKTI: [[&str; 9]; 3] = [
    [
        "recognition, authority and dealings with government, seniors or the father",
        "emotions, travel, the mother and contact with the public",
        "energy, property matters, courage and competition",
        "learning, trade, communication and friendships",
        "growth, wisdom, children, teachers and good fortune",
        "comforts, relationships, arts, vehicles and pleasures",
        "hard work, discipline, responsibility and some delays",
        "ambition, sudden opportunities and foreign connections",
        "detachment, spiritual interests and unexpected changes",
    ],
    [
        "அங்கீகாரம், அதிகாரம், அரசு, மேலதிகாரிகள், தந்தை தொடர்பான விஷயங்கள்",
        "உணர்வுகள், பயணம், தாய், பொதுமக்கள் தொடர்பு",
        "ஆற்றல், நிலம் சார்ந்த விஷயங்கள், தைரியம், போட்டி",
        "கல்வி, வணிகம், தொடர்பு, நட்பு",
        "வளர்ச்சி, ஞானம், குழந்தைகள், குரு, நல்வாய்ப்பு",
        "வசதி, உறவுகள், கலைகள், வாகனம், இன்பங்கள்",
        "கடின உழைப்பு, ஒழுக்கம், பொறுப்பு, சில தாமதங்கள்",
        "லட்சியம், திடீர் வாய்ப்புகள், வெளிநாட்டுத் தொடர்புகள்",
        "பற்றின்மை, ஆன்மீக ஆர்வம், எதிர்பாராத மாற்றங்கள்",
    ],
    [
        "मान-सम्मान, अधिकार और सरकार, वरिष्ठों या पिता से संबंधित विषय",
        "भावनाएँ, यात्रा, माता और जनसंपर्क",
        "ऊर्जा, भूमि-संपत्ति, साहस और प्रतिस्पर्धा",
        "शिक्षा, व्यापार, संवाद और मित्रता",
        "उन्नति, ज्ञान, संतान, गुरु और सौभाग्य",
        "सुख-सुविधा, संबंध, कला, वाहन और आनंद",
        "परिश्रम, अनुशासन, ज़िम्मेदारी और कुछ विलंब",
        "महत्वाकांक्षा, अचानक अवसर और विदेशी संबंध",
        "वैराग्य, आध्यात्मिक रुचि और अप्रत्याशित परिवर्तन",
    ],
];

const YOGA_RESULTS: [(&str, [&str; 3]); 21] = [
    ("Gaja Kesari", [
        "Gaja Kesari yoga gives intelligence, a good name and lasting prosperity; it also softens the effect of difficult combinations in the chart.",
        "கஜகேசரி யோகம் அறிவு, நற்பெயர், நிலையான செழிப்பைத் தரும்; ஜாதகத்தில் உள்ள சிரமமான சேர்க்கைகளின் தாக்கத்தையும் குறைக்கும்.",
        "गजकेसरी योग बुद्धि, यश और स्थायी समृद्धि देता है; यह कुंडली के कठिन योगों के प्रभाव को भी कम करता है।",
    ]),
    ("Budha Aditya", [
        "Budha Aditya yoga gives a bright, analytical mind, skill with words and recognition for intelligence.",
        "புதாதித்ய யோகம் கூர்மையான பகுப்பாய்வு அறிவு, சொல்வன்மை, அறிவுக்கான அங்கீகாரம் ஆகியவற்றைத் தரும்.",
        "बुधादित्य योग तेज़, विश्लेषणात्मक बुद्धि, वाक्पटुता और बुद्धि के लिए मान्यता देता है।",
    ]),
    ("Chandra Mangala", [
        "Chandra Mangala yoga gives enterprise and a talent for earning money, especially through business and property.",
        "சந்திர மங்கள யோகம் முயற்சியையும், குறிப்பாக வணிகம், சொத்து மூலம் பணம் ஈட்டும் திறனையும் தரும்.",
        "चंद्र-मंगल योग उद्यमशीलता और विशेषकर व्यापार व संपत्ति से धन कमाने की योग्यता देता है।",
    ]),
    ("Ruchaka", [
        "Ruchaka yoga makes you brave, strong and commanding, with success in leadership, defence, sports or land.",
        "ருசக யோகம் வீரம், வலிமை, ஆணையிடும் திறன் தந்து தலைமை, பாதுகாப்பு, விளையாட்டு, நிலம் ஆகியவற்றில் வெற்றி தரும்.",
        "रुचक योग साहसी, बलवान और प्रभावशाली बनाता है; नेतृत्व, रक्षा, खेल या भूमि में सफलता देता है।",
    ]),
    ("Bhadra", [
        "Bhadra yoga gives eloquence, learning and business skill, and a long, comfortable life.",
        "பத்ர யோகம் சொல்வன்மை, கல்வி, வணிகத் திறன், நீண்ட வசதியான வாழ்வைத் தரும்.",
        "भद्र योग वाक्पटुता, विद्या, व्यापार-कौशल और दीर्घ, सुखी जीवन देता है।",
    ]),
    ("Hamsa", [
        "Hamsa yoga gives a righteous, respected and spiritually inclined nature, with good fortune throughout life.",
        "ஹம்ச யோகம் நேர்மையான, மதிக்கப்படும், ஆன்மீக நாட்டமுள்ள இயல்பையும் வாழ்நாள் முழுவதும் நல்வாய்ப்பையும் தரும்.",
        "हंस योग धर्मपरायण, सम्मानित और आध्यात्मिक स्वभाव तथा जीवन भर सौभाग्य देता है।",
    ]),
    ("Malavya", [
        "Malavya yoga brings beauty, comforts, vehicles, artistic taste and a happy married life.",
        "மாளவ்ய யோகம் அழகு, வசதிகள், வாகனங்கள், கலை ரசனை, மகிழ்ச்சியான மணவாழ்வைத் தரும்.",
        "मालव्य योग सौंदर्य, सुख-सुविधा, वाहन, कलात्मक रुचि और सुखी वैवाहिक जीवन देता है।",
    ]),
    ("Sasa", [
        "Sasa yoga gives authority over people and organisations, discipline and success that grows with age.",
        "சச யோகம் மக்கள் மற்றும் அமைப்புகள் மீது அதிகாரம், ஒழுக்கம், வயதுடன் வளரும் வெற்றியைத் தரும்.",
        "शश योग लोगों और संस्थाओं पर अधिकार, अनुशासन और आयु के साथ बढ़ती सफलता देता है।",
    ]),
    ("Durudhara", [
        "Durudhara yoga gives wealth, generosity, vehicles and a comfortable, respected life.",
        "துருதுரா யோகம் செல்வம், தாராள குணம், வாகனங்கள், வசதியான மதிப்புமிக்க வாழ்வைத் தரும்.",
        "दुरुधरा योग धन, उदारता, वाहन और सुखी, सम्मानित जीवन देता है।",
    ]),
    ("Sunapha", [
        "Sunapha yoga gives self-earned wealth, intelligence and a good reputation.",
        "சுனபா யோகம் சுயமாக ஈட்டிய செல்வம், அறிவு, நற்பெயரைத் தரும்.",
        "सुनफा योग स्वअर्जित धन, बुद्धि और अच्छी प्रतिष्ठा देता है।",
    ]),
    ("Anapha", [
        "Anapha yoga gives good health, a pleasing personality and comforts in later life.",
        "அனபா யோகம் நல்ல ஆரோக்கியம், இனிய ஆளுமை, பிற்கால வசதிகளைத் தரும்.",
        "अनफा योग अच्छा स्वास्थ्य, आकर्षक व्यक्तित्व और बाद के जीवन में सुख देता है।",
    ]),
    ("Kemadruma", [
        "Kemadruma yoga can bring periods of loneliness or financial strain; Gaja Kesari yoga and strong planets in kendras soften it, and steady effort overcomes it.",
        "கேமத்ரும யோகம் தனிமை அல்லது பணநெருக்கடிக் காலங்களைத் தரலாம்; கஜகேசரி யோகமும் கேந்திரங்களில் வலுவான கிரகங்களும் இதைக் குறைக்கும், விடாமுயற்சி இதை வெல்லும்.",
        "केमद्रुम योग अकेलेपन या आर्थिक तंगी के दौर ला सकता है; गजकेसरी योग और केंद्रों में बलवान ग्रह इसे कम करते हैं, और निरंतर प्रयास इसे पार कर लेता है।",
    ]),
    ("Vesi", [
        "Vesi yoga gives truthfulness, balance and steady progress in life.",
        "வேசி யோகம் உண்மை, சமநிலை, வாழ்வில் சீரான முன்னேற்றத்தைத் தரும்.",
        "वेशि योग सत्यनिष्ठा, संतुलन और जीवन में स्थिर प्रगति देता है।",
    ]),
    ("Kala Sarpa", [
        "Kala Sarpa yoga brings ups and downs and delays at times, but also great determination; results improve markedly after the middle years.",
        "கால சர்ப்ப யோகம் சில நேரம் ஏற்ற இறக்கங்களையும் தாமதங்களையும் தரும்; ஆனால் பெரும் மன உறுதியையும் தரும்; நடுத்தர வயதுக்குப் பின் பலன்கள் நன்கு மேம்படும்.",
        "कालसर्प योग कभी-कभी उतार-चढ़ाव और विलंब लाता है, पर दृढ़ संकल्प भी देता है; मध्य आयु के बाद फल स्पष्ट रूप से सुधरते हैं।",
    ]),
    ("Vasumathi", [
        "Vasumathi yoga brings wealth that keeps growing through your own steady efforts.",
        "வசுமதி யோகம் உங்கள் சொந்த விடாமுயற்சியால் தொடர்ந்து வளரும் செல்வத்தைத் தரும்.",
        "वसुमति योग आपके निरंतर प्रयास से लगातार बढ़ने वाला धन देता है।",
    ]),
    ("Amala", [
        "Amala yoga gives a spotless reputation, goodwill and lasting prosperity through virtuous deeds.",
        "அமல யோகம் களங்கமற்ற புகழ், நல்லெண்ணம், நற்செயல்களால் நிலையான செழிப்பைத் தரும்.",
        "अमल योग निष्कलंक प्रतिष्ठा, सद्भावना और सत्कर्मों से स्थायी समृद्धि देता है।",
    ]),
    ("Adhi", [
        "Adhi yoga makes you a leader or person of authority, with comfort, health and victory over opponents.",
        "அதி யோகம் தலைமைப் பதவி அல்லது அதிகாரம், வசதி, ஆரோக்கியம், எதிரிகள் மீது வெற்றியைத் தரும்.",
        "अधि योग नेतृत्व या अधिकार, सुख, स्वास्थ्य और विरोधियों पर विजय देता है।",
    ]),
    ("Sakata", [
        "Sakata yoga brings fluctuating fortunes, like the turning of a cart wheel; patience and saving in good times keep life steady.",
        "சகட யோகம் வண்டிச் சக்கரம் போல ஏறி இறங்கும் அதிர்ஷ்டத்தைத் தரும்; பொறுமையும் நல்ல காலத்தில் சேமிப்பும் வாழ்வைச் சீராக்கும்.",
        "शकट योग गाड़ी के पहिये की तरह घूमता भाग्य लाता है; धैर्य और अच्छे समय में बचत जीवन को स्थिर रखते हैं।",
    ]),
    ("Parivartana", [
        "Parivartana (exchange) yoga ties the two houses together, so each strengthens the other's matters.",
        "பரிவர்த்தனை யோகம் இரு பாவங்களையும் இணைக்கிறது; ஒன்றின் விஷயங்களை மற்றொன்று வலுப்படுத்தும்.",
        "परिवर्तन योग दोनों भावों को जोड़ता है, जिससे प्रत्येक दूसरे के विषयों को बल देता है।",
    ]),
    ("Raja", [
        "Raja yoga, the union of a kendra lord and a trikona lord, brings status, authority and success, especially in their dasas.",
        "கேந்திர, திரிகோண அதிபதிகளின் இணைவான ராஜ யோகம் அந்தஸ்து, அதிகாரம், வெற்றியைத் தரும், குறிப்பாக அவர்களின் தசைகளில்.",
        "केंद्र और त्रिकोण के स्वामियों के मेल से बना राज योग पद, अधिकार और सफलता देता है, विशेषकर उनकी दशाओं में।",
    ]),
    ("Graha Malika", [
        "Several planets sharing one house concentrate your energy on that area of life; it becomes a major theme, for good and for challenge.",
        "பல கிரகங்கள் ஒரே பாவத்தில் இருப்பதால் அந்த வாழ்க்கைப் பகுதியில் உங்கள் ஆற்றல் குவியும்; அது நன்மையும் சவாலும் தரும் முக்கியக் கருவாகும்.",
        "कई ग्रहों का एक भाव में होना आपकी ऊर्जा को जीवन के उसी क्षेत्र पर केंद्रित करता है; वह शुभ और चुनौती दोनों रूपों में मुख्य विषय बन जाता है।",
    ]),
];

const REMEDY: [[[&str; 7]; 9]; 3] = [
    [
        ["Wear red, saffron or copper shades, especially on Sundays.", "Fast on Sundays.", "Wheat, jaggery, red cloth and copper.", "Red lotus or red flowers.", "Ruby (after expert advice).", "Surya; Lord Shiva.", "Om Hraam Hreem Hraum Sah Suryaya Namah"],
        ["Wear white and cream, especially on Mondays.", "Fast on Mondays.", "Rice, milk, white cloth and silver.", "White flowers such as jasmine.", "Pearl (after expert advice).", "Chandra; Goddess Parvati.", "Om Shraam Shreem Shraum Sah Chandraya Namah"],
        ["Wear red, especially on Tuesdays.", "Fast on Tuesdays.", "Red lentils, jaggery and red cloth.", "Red flowers.", "Red coral (after expert advice).", "Lord Murugan; Hanuman.", "Om Kraam Kreem Kraum Sah Bhaumaya Namah"],
        ["Wear green, especially on Wednesdays.", "Fast on Wednesdays.", "Green gram, green cloth and books.", "Tulasi and green leaves.", "Emerald (after expert advice).", "Lord Vishnu.", "Om Braam Breem Braum Sah Budhaya Namah"],
        ["Wear yellow, especially on Thursdays.", "Fast on Thursdays.", "Bengal gram, turmeric, yellow cloth and books.", "Yellow flowers.", "Yellow sapphire (after expert advice).", "Guru (Dakshinamurthy); your teachers.", "Om Graam Greem Graum Sah Gurave Namah"],
        ["Wear white or light blue, especially on Fridays.", "Fast on Fridays.", "White rice, curd, sugar, white silk and perfume.", "White flowers.", "Diamond (after expert advice).", "Goddess Lakshmi.", "Om Draam Dreem Draum Sah Shukraya Namah"],
        ["Wear dark blue or black, especially on Saturdays.", "Fast on Saturdays.", "Sesame, sesame oil, black cloth and iron.", "Blue or violet flowers.", "Blue sapphire (only after expert advice).", "Lord Shani; Hanuman; Ayyappa.", "Om Praam Preem Praum Sah Shanaischaraya Namah"],
        ["Wear smoky or dark blue shades.", "Fast on Saturdays.", "Black gram and a blanket.", "Blue flowers.", "Hessonite (after expert advice).", "Goddess Durga.", "Om Bhraam Bhreem Bhraum Sah Rahave Namah"],
        ["Wear grey or multicoloured clothes.", "Fast on Tuesdays.", "Horse gram and multicoloured cloth.", "Multicoloured flowers.", "Cat's eye (after expert advice).", "Lord Ganesha.", "Om Sraam Sreem Sraum Sah Ketave Namah"],
    ],
    [
        ["சிவப்பு, காவி, செம்பு நிற ஆடைகள், குறிப்பாக ஞாயிற்றுக்கிழமைகளில்.", "ஞாயிற்றுக்கிழமை விரதம்.", "கோதுமை, வெல்லம், சிவப்பு ஆடை, செம்பு.", "செந்தாமரை அல்லது சிவப்பு மலர்கள்.", "மாணிக்கம் (நிபுணர் ஆலோசனைக்குப் பின்).", "சூரிய பகவான்; சிவபெருமான்.", "ஓம் ஹ்ராம் ஹ்ரீம் ஹ்ரௌம் ஸஃ சூர்யாய நமஹ"],
        ["வெள்ளை, இளமஞ்சள் ஆடைகள், குறிப்பாகத் திங்கட்கிழமைகளில்.", "திங்கட்கிழமை விரதம்.", "அரிசி, பால், வெள்ளை ஆடை, வெள்ளி.", "மல்லிகை போன்ற வெள்ளை மலர்கள்.", "முத்து (நிபுணர் ஆலோசனைக்குப் பின்).", "சந்திர பகவான்; பார்வதி தேவி.", "ஓம் ஶ்ராம் ஶ்ரீம் ஶ்ரௌம் ஸஃ சந்த்ராய நமஹ"],
        ["சிவப்பு ஆடைகள், குறிப்பாகச் செவ்வாய்க்கிழமைகளில்.", "செவ்வாய்க்கிழமை விரதம்.", "துவரம் பருப்பு, வெல்லம், சிவப்பு ஆடை.", "சிவப்பு மலர்கள்.", "பவளம் (நிபுணர் ஆலோசனைக்குப் பின்).", "முருகப் பெருமான்; அனுமன்.", "ஓம் க்ராம் க்ரீம் க்ரௌம் ஸஃ பௌமாய நமஹ"],
        ["பச்சை ஆடைகள், குறிப்பாகப் புதன்கிழமைகளில்.", "புதன்கிழமை விரதம்.", "பச்சைப் பயறு, பச்சை ஆடை, புத்தகங்கள்.", "துளசி, பச்சை இலைகள்.", "மரகதம் (நிபுணர் ஆலோசனைக்குப் பின்).", "மகா விஷ்ணு.", "ஓம் ப்ராம் ப்ரீம் ப்ரௌம் ஸஃ புதாய நமஹ"],
        ["மஞ்சள் ஆடைகள், குறிப்பாக வியாழக்கிழமைகளில்.", "வியாழக்கிழமை விரதம்.", "கொண்டைக்கடலை, மஞ்சள், மஞ்சள் ஆடை, புத்தகங்கள்.", "மஞ்சள் மலர்கள்.", "புஷ்பராகம் (நிபுணர் ஆலோசனைக்குப் பின்).", "குரு (தட்சிணாமூர்த்தி); உங்கள் ஆசிரியர்கள்.", "ஓம் க்ராம் க்ரீம் க்ரௌம் ஸஃ குரவே நமஹ"],
        ["வெள்ளை அல்லது இளநீல ஆடைகள், குறிப்பாக வெள்ளிக்கிழமைகளில்.", "வெள்ளிக்கிழமை விரதம்.", "வெள்ளை அரிசி, தயிர், சர்க்கரை, வெள்ளைப் பட்டு, வாசனைப் பொருள்.", "வெள்ளை மலர்கள்.", "வைரம் (நிபுணர் ஆலோசனைக்குப் பின்).", "மகாலட்சுமி.", "ஓம் த்ராம் த்ரீம் த்ரௌம் ஸஃ ஶுக்ராய நமஹ"],
        ["கருநீலம் அல்லது கருப்பு ஆடைகள், குறிப்பாகச் சனிக்கிழமைகளில்.", "சனிக்கிழமை விரதம்.", "எள், நல்லெண்ணெய், கருப்பு ஆடை, இரும்பு.", "நீல அல்லது ஊதா மலர்கள்.", "நீலம் (நிபுணர் ஆலோசனைக்குப் பின் மட்டும்).", "சனி பகவான்; அனுமன்; ஐயப்பன்.", "ஓம் ப்ராம் ப்ரீம் ப்ரௌம் ஸஃ ஶனைஶ்சராய நமஹ"],
        ["புகை நிற அல்லது கருநீல ஆடைகள்.", "சனிக்கிழமை விரதம்.", "உளுந்து, போர்வை.", "நீல மலர்கள்.", "கோமேதகம் (நிபுணர் ஆலோசனைக்குப் பின்).", "துர்கை அம்மன்.", "ஓம் ப்ராம் ப்ரீம் ப்ரௌம் ஸஃ ராஹவே நமஹ"],
        ["சாம்பல் அல்லது பல வண்ண ஆடைகள்.", "செவ்வாய்க்கிழமை விரதம்.", "கொள்ளு, பல வண்ண ஆடை.", "பல வண்ண மலர்கள்.", "வைடூரியம் (நிபுணர் ஆலோசனைக்குப் பின்).", "விநாயகப் பெருமான்.", "ஓம் ஸ்ராம் ஸ்ரீம் ஸ்ரௌம் ஸஃ கேதவே நமஹ"],
    ],
    [
        ["लाल, केसरिया या ताम्र रंग के वस्त्र, विशेषकर रविवार को।", "रविवार का व्रत।", "गेहूँ, गुड़, लाल वस्त्र और ताँबा।", "लाल कमल या लाल पुष्प।", "माणिक्य (विशेषज्ञ की सलाह के बाद)।", "सूर्य देव; भगवान शिव।", "ॐ ह्रां ह्रीं ह्रौं सः सूर्याय नमः"],
        ["सफ़ेद और क्रीम रंग के वस्त्र, विशेषकर सोमवार को।", "सोमवार का व्रत।", "चावल, दूध, सफ़ेद वस्त्र और चाँदी।", "चमेली जैसे सफ़ेद पुष्प।", "मोती (विशेषज्ञ की सलाह के बाद)।", "चंद्र देव; माँ पार्वती।", "ॐ श्रां श्रीं श्रौं सः चंद्रमसे नमः"],
        ["लाल वस्त्र, विशेषकर मंगलवार को।", "मंगलवार का व्रत।", "मसूर दाल, गुड़ और लाल वस्त्र।", "लाल पुष्प।", "मूँगा (विशेषज्ञ की सलाह के बाद)।", "कार्तिकेय; हनुमान जी।", "ॐ क्रां क्रीं क्रौं सः भौमाय नमः"],
        ["हरे वस्त्र, विशेषकर बुधवार को।", "बुधवार का व्रत।", "मूँग, हरा वस्त्र और पुस्तकें।", "तुलसी और हरी पत्तियाँ।", "पन्ना (विशेषज्ञ की सलाह के बाद)।", "भगवान विष्णु।", "ॐ ब्रां ब्रीं ब्रौं सः बुधाय नमः"],
        ["पीले वस्त्र, विशेषकर गुरुवार को।", "गुरुवार का व्रत।", "चना दाल, हल्दी, पीला वस्त्र और पुस्तकें।", "पीले पुष्प।", "पुखराज (विशेषज्ञ की सलाह के बाद)।", "बृहस्पति देव; अपने गुरुजन।", "ॐ ग्रां ग्रीं ग्रौं सः गुरवे नमः"],
        ["सफ़ेद या हल्के नीले वस्त्र, विशेषकर शुक्रवार को।", "शुक्रवार का व्रत।", "सफ़ेद चावल, दही, चीनी, सफ़ेद रेशम और इत्र।", "सफ़ेद पुष्प।", "हीरा (विशेषज्ञ की सलाह के बाद)।", "माँ लक्ष्मी।", "ॐ द्रां द्रीं द्रौं सः शुक्राय नमः"],
        ["गहरे नीले या काले वस्त्र, विशेषकर शनिवार को।", "शनिवार का व्रत।", "तिल, तिल का तेल, काला वस्त्र और लोहा।", "नीले या बैंगनी पुष्प।", "नीलम (केवल विशेषज्ञ की सलाह के बाद)।", "शनि देव; हनुमान जी।", "ॐ प्रां प्रीं प्रौं सः शनैश्चराय नमः"],
        ["धूसर या गहरे नीले वस्त्र।", "शनिवार का व्रत।", "उड़द और कंबल।", "नीले पुष्प।", "गोमेद (विशेषज्ञ की सलाह के बाद)।", "माँ दुर्गा।", "ॐ भ्रां भ्रीं भ्रौं सः राहवे नमः"],
        ["स्लेटी या बहुरंगी वस्त्र।", "मंगलवार का व्रत।", "कुलथी और बहुरंगी वस्त्र।", "बहुरंगी पुष्प।", "लहसुनिया (विशेषज्ञ की सलाह के बाद)।", "भगवान गणेश।", "ॐ स्रां स्रीं स्रौं सः केतवे नमः"],
    ],
];
