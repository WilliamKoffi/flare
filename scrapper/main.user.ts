// ==UserScript==
// @name         Lawyer Directory Extractor
// @namespace    http://tampermonkey.net/
// @version      2.0
// @description  Extracts and downloads directory data to TOML
// @author       The Affordance Architect
// @match        https://app.ordredesavocats-ci.net/annuaire*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=ordredesavocats-ci.net
// @grant        none
// ==/UserScript==

import * as Board from "./src/board";

Board.inject();
