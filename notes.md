# Notes

## Post-import TODO

Must have:
+ popup edit game menu
+ session star reviews
+ duplicate management
    + search for existing duplicates
    + option to delete game in edit menu
    + when adding, try to catch duplicate entries
+ separate pass options:
    + pass (wait for sale) -> auto longer wait period
    + pass (unreleased) -> auto longer wait period
+ separate UI presentation for tags and ownership
+ screen to show EVERY game in DB in compact form
+ retro tag, new tag update screen
+ zip/otherwise compress old version of the database.json (since it gets quite large)
+ separate "no release date" option into:
    + no release date (unreleased)
    + no release date (released)
    + and fix data

Nice to have:
+ details screen show session history
+ go through TODOs in code
+ stats screen burndown chart
+ handle back button
+ Progress bar for IGDB updates

Way later:
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