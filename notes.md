# Notes

## Next step
Next step is probably to proof of concept using google docs or sheets as a storage backend...
Actually, do I need to do this? If I'm running this on my NAS, I can store the DB in plaintext
without issue and back it up however I like... why hit another server?

## IGDB status
Now that webasm -> rocket server -> IGDB is working, there are improvements to be made:
+ not getting a completely new bearer token for every API request
? Not storing secret on the server, bearer token only?
? Update bearer token from secure machine? Or server can refresh itself?

## Problems with the serverless approach

### Goals/Requirements
1. Usable from my phone and my PC
2. Use the Twitch API, I just like the data from it
3. Ideally serverless (actually serverless)

(3) is where all the issues I've had so far come in. Being serverless means I have to access at
least two external APIs from my webasm context in a browser: Some games API, and some synced
storage API. This means running into auth and CORS problems.

If I give up on (3), I can still write everything in Rust and share data. I just also need to run
a server. But it will make CORS go away, and limit what I have to test in the browser.