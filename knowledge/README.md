# Vedic Astrology knowledge folder

This folder is the app's **Jyotisha knowledge base**. It is used in two ways:

1. **Rule engine (always on, no AI needed).** The files under `rules/<language>/` are structured
   interpretation rules. The report evaluates them against the calculated chart (for example "lord
   of house 2 is in house 5") and prints the matching texts, with the rule id, so every prediction
   can be traced back to its rule.
2. **Reference library for the optional AI writer (RAG).** When an OpenAI-compatible model is set
   up in the app (Full report tab, "AI writer"), the app searches every `.md`, `.txt` and `.json`
   file in this folder for passages that match each report section (planet names, houses, topics)
   and sends the best ones to the model with the chart facts. The model only expands and explains
   what the rules already found; it never calculates positions.

The app copies this folder to `~/.vedic_astrology/knowledge` on first start. Point the app at another
folder in the AI writer settings to use your own library.

## Layout

```
knowledge/
  README.md            this file
  sources.json         register of classical books, editions and rights
  rules/
    en/ ta/ hi/        one folder per language
      lord_in_house.json   144 rules: lord of house N placed in house M (BPHS ch. 24)
      dasha.json           Vimshottari mahadasha readings: general / strong / weak
  texts/               put source passages here (verses, translations, notes) for the AI writer
```

## Rule file format (`vedic-astrology-rules/1`)

```json
{
  "schema": "vedic-astrology-rules/1",
  "file": "lord_in_house",
  "language": "en",
  "source": {"book": "...", "chapter": "...", "edition": null, "note": "..."},
  "status": "paraphrase_unverified",
  "rules": [ {"id": "BHAVESH-02-05", "lord_of": 2, "placed_in": 5, "text": "..."} ]
}
```

* `status` tells you how much to trust a rule: `paraphrase_unverified` (classical meaning in plain
  words, not yet checked against a named edition), `verified` (checked against `source.edition`,
  chapter and verse), `illustrative` (example only).
* Keep the same `id` across languages so translations stay linked.
* Edit a text here and the next report uses it; ids the app doesn't know are ignored.

## Adding classical texts for the AI writer

Put plain-text or Markdown files in `texts/`, one topic per file, for example
`texts/bphs_ch24_house_lords.md`. Start each file with a short header saying the book, edition,
translator and licence. The search works on words, so transliterated Sanskrit terms (bhava,
lagnesha, dasha) and English names both help.

**Rights.** The age of a Sanskrit work does not make a modern translation, transcription or
commentary free to reuse. Record the edition, translator and redistribution rights of every source
in `sources.json` before adding its text.

**Accuracy.** These readings describe traditional interpretations. They are not scientifically
validated predictions of future events.

## Sanskrit knowledge (`sanskrit/`)

* `glossary.json`: lists the Sanskrit name (in Latin letters and Devanagari) of the 12 bhavas, 9 grahas, 12 rashis and 27 nakshatras.
  * Bhavas also carry their karakas (significators) and kind (kendra, trikona, dusthana, upachaya, maraka…).
  * Grahas also carry their significations and Vimshottari years.
  * Rashis also carry their lords, elements and modality.
  * Nakshatras also carry their deities and dasha lords.
* The report prints each house's Sanskrit name, karakas and kind under its heading.
* `concepts.md`: plain-language explanations of the Sanskrit terms the report uses (graha, bhava, drishti, yoga, dosha, dasha, nakshatra, panchanga, upaya). The AI writer can retrieve them.

Both are terminology written for this app. They quote no verses or translations. To add original Sanskrit verses, put them in `texts/` with their edition and rights recorded in `sources.json`.

New built-in files are added to an existing knowledge folder on the next start. Files you have edited are never overwritten.

## AI instructions (`ai/instructions.md`)

These are the instructions the AI writer follows. Edit the file in your knowledge folder (⚙ Settings → Edit AI instructions) to change what it writes. Keep its two headings:

* "Report sections" is used when expanding the full report's house and dasa readings.
* "Questions" is used by the "Ask a question about this chart" box.

`{lang}` becomes English, Tamil or Hindi. The search never returns this file as a reference passage.

## Classical texts (`texts/`)

⚙ Settings → Classical texts → Download fills `texts/jyotisha-corpus/` from https://github.com/maximally0/jyotisha-corpus. Command line: `vedic_astrology --import-texts <knowledge folder>`.

* **What arrives:** public-domain editions (pre-1929) of Brihat Jataka, Brihat Samhita, Saravali, Laghu Parashari, Surya Siddhanta, Panchasiddhantika and The Hindu Book of Astrology, plus the corpus's extracted rule records.
* **Licence:** the corpus data is CC BY 4.0; the attribution is in `texts/SOURCES.md`.
* **Personal-study option:** a checkbox (or `--personal` on the command line) also fetches the Brihat Parashara Hora Shastra chapters from indike.org into `texts/personal/`.
  * indike.org's Sanskrit text comes from sanskritdocuments.org, whose terms allow personal study and research only.
  * That folder is for your own use: never share it or package it. The Docker image leaves it out.
