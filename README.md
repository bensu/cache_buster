# cache_buster

A command line tool (TODO: and library) to fingerprint file names with their hash contents to aid with HTTP caching.

## The Problem

1. You have a web server.
2. You serve static files like `css/main.css`, `js/compiled-output.js`, or `img/logo.svg`
3. You sometimes (but rarely) change those files so you don't know how long should browsers cache them.

## The solution

1. You extract a hash of the file contents and you make a copy of the file that has that hash in the name (fingerprinted).
2. You link to and serve the fingerprinted file from your HTML. `<script src="js/compiled-output.FINGERPRINT.cached.js">`
3. You tell browsers to cache the fingerprinted forever `Cache-Control: max-age=31556926`
4. When you change the original file, you'll get a different fingerprint and the browser won't ever be confused.

This is only one step (fingerprinting) of the many steps the [Rails Asset Pipeline](http://guides.rubyonrails.org/asset_pipeline.html) has.

## Use

Make a configuration file with the following options:

```
{
  // all the options should be under the "cache_buster" key
  "cache_buster": {

    // glob patterns for which files should be fingerprinted
    "patterns": ["resources/public/*.js", "resources/public/*.css", "resources/public/*/*.txt"],

    // the root of the directory where the files will be served from
    "asset_path": "resources/public",

    // (optional) a string that will be added to the fingerprinted file to help `clean` find it (default "cached")
    "marker": "cached",

    // the path where cache_buster will put the mapping from file -> fingerprinted_copy
    "dictionary": "resources/asset-manifest.json"
  }
}
```

Add all the fingerprinted copies with `fingerprint`:

```
cache_buster fingerprint package.json
```

Remove all the fingerprinted files with `clean`:

```
cache_buster clean package.json
```

The code that runs your web server and creates your HTML pages now needs to know the names of the fingerprinted files. That is why `cache_buster` makes a _manifest file_. The JSON manifest file maps the original file names to the fingerprinted ones:

```
// asset_mainifest.json
{
 "css/main_file.css": "css/main.C0F781B05E475681EAF474CB242F.cached.css",
 "js/compiled-output.js": "js/compiled-output.D41D8CD98F0B24E980998ECF8427E.cached.js",
 "img/logo.txt":"img/logo.AFE9EC29D0DF67ABACB95AFECC6B26B.cached.txt"
}
```

From Rust:

TODO

From Clojure:

```clj
(require '[clojure.data.json :as json])
(require '[clojure.java.io :as io])
(require '[hiccup.core :refer [html]])
(require '[compojure.core :as compo])

;; read the manifest file
(def path->fingerprint
  (json/read-str (slurp (io/resource "asset-manifest.json"))))

;; use the fingerprinted file from the html
(defn js-tags []
  (html
   [:script {:src (path->fingerprint "js/compiled-output.js")}]))

;; add Cache-Control max-age=31556926 when the file is fingerprinted, no-cache otherwise
(defn wrap-cache-control [handler]
  (let [marker-string ".cached."]
    (fn [request]
      (if (.contains (:uri request) marker-string)
        (some-> (handler request)
                (assoc-in [:headers "Cache-Control"] "max-age=31556926"))
        (some-> (handler request)
                (assoc-in [:headers "Pragma"] "no-cache")
                (assoc-in [:headers "Cache-Control"] "no-cache, no-store, must-revalidate")
                (assoc-in [:headers "Expires"] "0"))))))

;; serve all the static files from the asset-path
(def content-routes
  (let [asset-path "resources/public"]
    (wrap-cache-control
      (compo/routes
        (route/resources "/" {:root asset-path})))))
```

## Installation

From source:

```sh
# Clone this repository
git clone https://github.com/bensu/cache_buster

# Have Rust and Cargo installed
cargo build --release

# Move the binary to somewhere in your PATH (might need sudo)
mv target/release/cache_buster /usr/local/bin/
```

## License

All the code in this repository is released under the ***Mozilla Public License v2.0***, for more information take a look at the [LICENSE] file.
