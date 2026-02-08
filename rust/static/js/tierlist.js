function setup_tierlist() {
    var tiers = document.querySelectorAll(".tier");
    console.log()
    tiers.forEach(function(tier) {
        console.log(tier);
        setup_tier(tier);
    });

    var characters = document.querySelectorAll(".character-icon");
    characters.forEach(function(character) {
        character.draggable = true;
        character.ondragstart = char_ondragstart;
    });

    add_tier();
}

function setup_tier(tier) {
        tier.ondrop = tier_ondrop;
        tier.ondragover = tier_ondragover;

        tier_kill_dragover_for_name(tier);
}

function tier_ondrop(event) {
    event.preventDefault();
    const id = event.dataTransfer.getData("text/plain");
    const draggedElement = document.getElementById(id);

    const holder = event.currentTarget.querySelector(".holder");
    if (holder && draggedElement) {
        holder.appendChild(draggedElement);
    }
}


function tier_ondragover(event) {
    event.preventDefault();
    // TODO - Visually show dragover
}

function char_ondragstart(event) {
    event.dataTransfer.setData("text/plain", event.target.id);
}

function tier_kill_dragover_for_name(tier) {
    let name_div = tier.querySelector("name");
    if (name_div == undefined) return;
    
    name_div.addEventListener("dragover", (event) => {
        event.preventDefault();  // prevents default browser behavior
        event.stopPropagation(); // prevents interfering with tierlist
    });
    
    name_div.addEventListener("drop", (event) => {
        event.preventDefault();  // disable dropping
        event.stopPropagation();
    });
}

// Creates and returns a tier.
function generate_tier(text="new tier", color="orange") {
    let new_tier = document.createElement("div");
    new_tier.classList.add("tier");

    let tier_settings = document.createElement("div");
    tier_settings.classList.add("settings");
    tier_settings.innerHTML = `<img src="/static/img/ui/up-arrow.png" onclick="move_tier_up(this)">
                <input type="color" value="#ff0000" oninput="colorpicker_oninput(this)">
                <img src="/static/img/ui/down-arrow.png" onclick="move_tier_down(this)">`;

    let tier_name = document.createElement("div");
    tier_name.classList.add("name");
    tier_name.contentEditable = true;
    tier_name.innerHTML = text;
    tier_name.style.backgroundColor = color;
    tier_name.spellcheck = false;

    let tier_holder = document.createElement("div");
    tier_holder.classList.add("holder");

    new_tier.append(tier_settings, tier_name, tier_holder);

    setup_tier(new_tier);
    return new_tier;
}

// Fired by color-pickers; changes the color of the relevant tier name.
function colorpicker_oninput(caller) {
    let parent_tier = caller.closest(".tier");
    let name_div = parent_tier.querySelector(".name");
    let color = caller.value;
    name_div.style.backgroundColor = color;
    
    let brightness = getBrightness(color);
    console.log(brightness);
    if (brightness > 0.6) {
        name_div.style.color = "black";
    }
    else if (brightness < 0.5) {
        name_div.style.color = "white";
    }
}

// Creates and adds a new tier at the bottom of the current ones
function add_tier() {
    let new_tier = generate_tier();
    document.getElementById("tierlist-holder").appendChild(new_tier);
}

function move_tier_up(calling_button) {
    let tier = calling_button.closest(".tier");
    let prev_tier = tier.previousElementSibling;
    if (prev_tier) {
        tier.parentNode.insertBefore(tier, prev_tier);
    }
}

function move_tier_down(calling_button) {
    let tier = calling_button.closest(".tier");
    let next_tier = tier.nextElementSibling;
    if (next_tier) {
        tier.parentNode.insertBefore(next_tier, tier);
    }
}

function delete_last_tier() {
    let deleted_tier = document.getElementById("tierlist-holder").lastChild;
    let rescuable_characters = deleted_tier.querySelectorAll(".character-icon");
    let tierlist_origin = document.getElementById("tierlist-origin").querySelector(".holder");
    tierlist_origin.append(...rescuable_characters);
    deleted_tier.remove();
}

function download_screenshot_of_tierlist() {
    const target = document.getElementById("tierlist-collector");
    if (!target) {
        console.warn(`Element with ID "tierlist-collector" not found.`);
        return;
    }

    const runCapture = () => {
        target.classList.add("hide-settings");
        html2canvas(target).then(canvas => {
        const image = canvas.toDataURL('image/png');
        target.classList.remove("hide-settings");
        const link = document.createElement('a');
        link.href = image;
        title = document.getElementById("tierlist-title").innerHTML.toLowerCase().replaceAll(" ", "-");
        link.download = `pd-tierlist-${title}`;
        link.click();
        });
    };

    // Load html2canvas if not already present
    if (window.html2canvas) {
        runCapture();
    } else {
        const script = document.createElement('script');
        script.src = 'https://cdn.jsdelivr.net/npm/html2canvas@1.4.1/dist/html2canvas.min.js';
        script.onload = runCapture;
        script.onerror = () => alert('Failed to load screenshot tool.');
        document.head.appendChild(script);
    }
}

// If retired characters are hidden, shows them. Otherwise, hides them. Updates button accordingly.
function flip_retired_characters(caller) {
    let document_wrapper = document.querySelector(".wrapper");
    if (document_wrapper.classList.contains("hide-retired")) {
        document_wrapper.classList.remove("hide-retired");
        caller.classList.remove("dark");
        caller.innerHTML = "Hide Retired Characters";
    }
    else {
        document_wrapper.classList.add("hide-retired");
        caller.classList.add("dark");
        caller.innerHTML = "Show Retired Characters";
    }
}

// If hidden characters are hidden, shows them. Otherwise, hides them. Updates button accordingly.
function flip_hidden_characters(caller) {
    let document_wrapper = document.querySelector(".wrapper");
    if (document_wrapper.classList.contains("hide-hidden")) {
        document_wrapper.classList.remove("hide-hidden");
        caller.classList.remove("dark");
        caller.innerHTML = "Hide Hidden Characters";
    }
    else {
        document_wrapper.classList.add("hide-hidden");
        caller.classList.add("dark");
        caller.innerHTML = "Show Hidden Characters";
    }
}

function getBrightness(color) {
    let r, g, b;

    if (color.startsWith('#')) {
        // Remove # and convert shorthand like #abc to full
        color = color.slice(1);
        if (color.length === 3) {
            color = color.split('').map(c => c + c).join('');
        }

        r = parseInt(color.slice(0, 2), 16);
        g = parseInt(color.slice(2, 4), 16);
        b = parseInt(color.slice(4, 6), 16);
    } else if (color.startsWith('rgb')) {
        // Extract numbers from rgb() or rgba()
        const matches = color.match(/\d+/g);
        [r, g, b] = matches.map(Number);
        } else {
        throw new Error('Unsupported color format');
    }

    // Brightness formula
    return (0.299 * r + 0.587 * g + 0.114 * b)/255;
}