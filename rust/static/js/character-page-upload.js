// Image files here are represented with an object, bc we need to discriminate between images
// already uploaded to the DB and those that are not. they all have the "state" property,
// which can be "uploaded" or "local". "local" means there's another property named "file"
// pointing to the local file that needs to be updated. "uploaded" means there's another
// property called "key" pointing to the image's current URL.

// Initialize only if not already defined in the page
if (typeof characterImageFiles === 'undefined') {
    // An object with the following properties that are built throughout:
    // thumbnail, logo, pageImg
    characterImageFiles = {};
}

const containers = {
    thumbnail: document.getElementById("characterThumbnail"),
    logo: document.getElementById("characterLogo"),
    pageImg: document.getElementById("characterImage")
}

Object.keys(characterImageFiles).forEach((propertyName) => {
    const img = document.createElement('img');
    img.src = characterImageFiles[propertyName].key;
    containers[targetImageKey].appendChild(img);
});

/// Given text, puts it in the Error Text div.
function updateErrorText(text) {
    document.getElementById("errorDisplay").innerHTML = text;
}

// Ran when the user selects a new image file.
function setImageFile(event, targetImageKey) {
    const file = event.target.files[0];

    // We assume it's one of the allowed filetypes. If you broke it, not my fuckin problem, this is the client.

    if (file) {
        characterImageFiles[targetImageKey] = {
            state: "local",
            file: file
        };

        // Now, make the user see the new thumbnail listed.
        const reader = new FileReader();

        reader.onload = (e) => {
            // Create an img element
            const img = document.createElement('img');
            img.src = e.target.result; // This is the base64 data URL

            // Clear out any existing thumbnail visual.
            containers[targetImageKey].innerHTML = '';
            // Add it to the page
            containers[targetImageKey].appendChild(img);
        };

        reader.readAsDataURL(file); // Read as data URL for images
        event.target.value = '';
    }
}