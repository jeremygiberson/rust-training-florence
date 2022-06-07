# Florence
Create an HTTP server with the familiarity* of express.js. 

It's fast**, superfast***, like [Florence Griffith Joyner](https://en.wikipedia.org/wiki/Florence_Griffith_Joyner).


##### Disclaimers
* Project created as part of a Rust learning exercise, it is neither feature rich nor feature complete.
* \* claims of likeness to other libraries merely indicate source of influence, does not confer any guarantee of actual similarity. 
* **, ** claims have not been tested or verified.


## Features
* [x] Supports limited HTTP 1.0 protocol
  * things like multipart requests not supported
* [x] Express like route definition interface (Ie. `method('route', handler(){})`. The following methods have been implemented:
  * [x] GET
  * [ ] remaining as necessary
* [ ] Route parameter support
* [ ] Multithreaded request handling
* [ ] Middlewares (Stretch goal)