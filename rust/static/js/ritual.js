var character_is_revealed = false;
var character_list;
var current_character;

function ritualV3Button() {
    if (!character_is_revealed) {
        console.log("Character not revealed, revealing.");
        character_is_revealed = true;
        ritualV3Reveal();
    }
    else {
        console.log("Progressing in ritual.");
        character_is_revealed = false;
        ritualV3Progress();
    }
}

function ritualV3Reveal() {
    var nameTag = document.getElementById("ritual-v3-name");
    var powerTag = document.getElementById("ritual-v3-power");
    var buttonTag = document.getElementById("ritual-v3-button");
    var powerExplanationTag = document.getElementById("ritual-v3-power-desc");

    nameTag.innerHTML = current_character["name"];
    powerTag.innerHTML = current_character["power"];
    powerExplanationTag.innerHTML = current_character["powerDesc"];
    powerExplanationTag.style.display = "block";
    buttonTag.innerHTML = "NEXT →";
}

function ritualV3Progress() {
    console.log("Loading next character, " + character_list.length + " remaining.");

    if (character_list.length > 0) {
        current_character = character_list.pop();

        var nameTag = document.getElementById("ritual-v3-name");
        var artTag = document.getElementById("ritual-v3-art");
        var powerTag = document.getElementById("ritual-v3-power");
        var buttonTag = document.getElementById("ritual-v3-button");
        var powerExplanationTag = document.getElementById("ritual-v3-power-desc");
        var guessedCharactersTag = document.getElementById("ritualDone");

        artTag.src = "/assets/img/pd-logo.svg"; // To make the image visibly changed.
        artTag.src = current_character["img"];
        nameTag.innerHTML = "NAME?";
        powerTag.innerHTML = "POWER?";
        powerExplanationTag.innerHTML = "???";
        powerExplanationTag.style.display = "none";
        buttonTag.innerHTML = "REVEAL";

        guessedCharactersTag.innerHTML = parseInt(guessedCharactersTag.innerHTML) + 1;
    }
    else {
        console.log("Finished ritual!");
        window.location.href = new URL("/misc/ritual/complete", window.location.href).href;
    }
}

function initializeRitual(list_of_characters = {}) {
    character_list = list_of_characters;

    shuffle(character_list);

    var totalRemainingTag = document.getElementById("ritualTotal");
    totalRemainingTag.innerHTML = character_list.length;

    ritualV3Progress();
}

function shuffle(array) {
    console.log("Starting shuffle");
    let currentIndex = array.length;
    // While there remain elements to shuffle...
    while (currentIndex != 0) {
        // Pick a remaining element...
        let randomIndex = Math.floor(Math.random() * currentIndex);
        currentIndex--;
        // And swap it with the current element.
        [array[currentIndex], array[randomIndex]] = [
            array[randomIndex], array[currentIndex]];
    }
}