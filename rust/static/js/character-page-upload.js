// Image files here are represented with an object, bc we need to discriminate between images
// already uploaded to the DB and those that are not. they all have the "state" property,
// which can be "uploaded" or "local". "local" means there's another property named "file"
// pointing to the local file that needs to be updated. "uploaded" means there's another
// property called "key" pointing to the image's current URL.

// Initialize only if not already defined in the page
if (typeof filesInImageContainer === 'undefined') {
    // An object with the following properties that are built throughout:
    // thumbnail, logo, pageImg
    characterImageFiles = {};
}

/// Given text, puts it in the Error Text div.
function updateErrorText(text) {
    document.getElementById("errorDisplay").innerHTML = text;
}