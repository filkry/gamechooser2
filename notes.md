# Notes

## Goals/Requirements
1. Usable from my phone and my PC
2. Use the Twitch API, I just like the data from it
3. Ideally serverless (actually serverless)

(3) is where all the issues I've had so far come in. Being serverless means I have to access at
least two external APIs from my webasm context in a browser: Some games API, and some synced
storage API. This means running into auth and CORS problems.

If I give up on (3), I can still write everything in Rust and share data. I just also need to run
a server. But it will make CORS go away, and limit what I have to test in the browser.