# Notes

## Post-import TODO

Must have:
+ when retiring, allow marking game as ignore passes

Nice to have:
+ details screen show session history
+ go through TODOs in code
+ when adding, try to catch duplicate entries
+ stats screen burndown chart
+ periodically refresh data from IGDB
+ add star ratings when closing sessions (optional)
+ handle back button
+ standardize game display so that we always have the relevant preview information

Way later:
+ keep DB in memory while server is running
+ zip/otherwise compress old version of the database.json (since it gets quite large)
+ release webasm is like 1/4 the size, make some easy way to build/deploy as release instead
+ Strengthen auth to where I could expose this outside local network
+ even better error handling in the client
+ details screen embed additional stuff from IGDB
+ edit screen new own add/remove UI

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