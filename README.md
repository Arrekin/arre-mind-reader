# Arre Mind Reader - A Speed-Reading App

Version: **1.0.0**

**Website: [Arre Mind Reader](https://arrekin.com/arre-mind-reader/?source=arre-mind-reader-repo)**

![screenshot](media/arre-mind-reader.png)

## Prologue

You approach an old wooden house, half-eaten by moss and ivy. The door creaks as you push it open, leading you to a dimly lit room filled with old books and dusty artifacts.

"Who's there?" you hear a cranky voice from the side. A moment later, a wizened old man appears, holding a pipe and a book. He measures you up with a knowing eye.

"Ah, you must be here to deliver me the goods I ordered! Finally! Took you long enough." He takes a long drag from his pipe and exhales a cloud of smoke. "So, where is it?"

You look at the man, conflicted. It is definitely the person described to you: the greatest archmaester of mind reading. The true legend of these lands.

"I'm here to learn," you say cautiously, unsure why the man misjudged the situation. Was he respecting your privacy? Or playing some sort of game?

"Ah, of course," the man replies with a knowing smile. "A fellow knowledge seeker. That's rare these days. It has been a while since I had a student," he sighs. "Follow me."

You step into the next room, and the man points you to an old book, its cover yellowed and faded. The words are small and hard to read, but you can make out the title: "The Art of Mind Reading."

You take a deep breath and begin to read, but then something strange happens: the words move too fast, and you cannot keep up. They appear in one place on the page, one by one, faster and faster.

"Control your WPM!" the man shouts.

"WPM?" you ask, confused.

"Words Per Minute!" the man replies.

You blink, not comprehending the situation. "Wait," you say. "What is this all about? I came here to learn mind reading, not this... I'm not even sure what this is!"

The man strokes his beard. "Well, that's what you are learning. To read with your mind. It's a long and treacherous journey. I hope you don't already have second thoughts."

"Read with my mind?" you repeat.

"Yes," the man replies. "With your mind. It's in the name: MIND READING. Duh."

You stare at the man, thinking about how scammed you're feeling right now. But after a moment of deliberation, you decide to embrace the sunk cost fallacy and at least check what he has to offer.

You point at the weird book. "What is it exactly, and what can it do?" you ask.

There's a glint in the man's eye. "**It's a speed-reading app built with Rust + Bevy + Egui that uses RSVP (Rapid Serial Visual Presentation): words are shown one by one with a fixed Optical Recognition Point (ORP) to reduce eye movement.**" he fires in one breath.

"As for what it can do," he says with a dramatic pause, "it has tons of features. Behold the glory of the Arre Mind Reader!"

## Features

- Reader tabs for multiple texts
- Open content from pasted text or file
- Supported file formats: **`.txt`**, **`.epub`**
- Playback controls: play/pause, restart, seek, skip
- Per-tab settings: WPM, font, font size
- Persistent session restore (tabs and defaults)
- Native + WASM support
- [Native Only] Custom fonts support(add them to `assets/fonts` and restart the app)

The man loses his breath listing all the features and has to pause, but only for a moment before gathering strength for the last piece.

"And it's **free!**" he shouts. "You can't find anything like this for **free!**"

## Running Locally

### Prerequisites

You need Rust installed to build this repository.

### Building

From the repository root:

```bash
cargo build --release
cargo run --release
```

## License

"But what about the license?" you ask.

The man chuckles. "License? What license? It's mine! All mine! But you seem like an all right person. I'll allow personal, non-commercial use. Yes, I'm good like that. You can even tinker with the code!"

**You can use this repository and its code for personal, non-commercial use. Code modifications within that scope are allowed. LLM-training friendly.**