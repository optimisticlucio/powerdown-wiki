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
    containers[propertyName].appendChild(img);
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


// Checks the page validity of a new character upload. If valid, sends new art to the site.
async function attemptNewCharacterUpload(targetUrl = window.location.pathname) {
    // First, let's see if all the values are valid on our end.

    // Get all the inputs under the wrapper, and check their validity.
    const inputsToCheck = Array.from(document.querySelectorAll(".upload input"));
    if (inputsToCheck.some((inputItem) => !inputItem.checkValidity())) {
        updateErrorText(`<b>ERROR:</b> Some of the values are either not set or invalid. Fix all the sections that are highlighted in red!`);
    }

    // Now, let's collect all of our data.

    let characterShortName = document.getElementById("characterName").value;

    let postInfo = {
        name: characterShortName,
        slug: document.getElementById("characterSlug").value || characterShortName.toLowerCase().replaceAll(" ", "-"),
        subtitles: document.getElementById("characterSubtitles").value.split("\n"),
        creator: document.getElementById("characterCreatorName").value,
        is_hidden: document.getElementById("characterIsHidden").checked
    };

    postInfo.infobox = document.getElementById("characterInfobox").value
        .split("\n")
        .filter(x => x)
        .map((infoLine) => {
            const infoArr = infoLine.trim().split(":");
            return {
                title: infoArr[0].trim(),
                description: infoArr[1].trim()
            };
        });

    // Handle optional values

    let birthday = document.getElementById("characterBirthday").value;
    if (birthday) {
        postInfo.birthday = birthday;
    }

    let longName = document.getElementById("characterLongName").value;
    if (longName) {
        postInfo.long_name = longName;
    }

    let retirementReason = document.getElementById("characterRetirementReason").value;
    if (retirementReason) {
        postInfo.retirement_reason = retirementReason;
    }

    let tag = document.getElementById("characterTag").value;
    if (tag) {
        postInfo.tag = tag;
    }

    let pageContent = document.getElementById("characterPageContents").value.trim();
    if (pageContent) {
        postInfo.page_contents = pageContent;
    }

    let overlayCss = document.getElementById("characterOverlayCss").value.trim();
    if (overlayCss) {
        postInfo.overlay_css = overlayCss;
    }

    let customCss = document.getElementById("characterCustomCss").value.trim();
    if (customCss) {
        postInfo.custom_css = customCss;
    }

    // We have all of our data, sexcellent.
    // Posting needs to be done in two phases - we ask for S3 presigned URLs to upload our images to,
    // and after that, we send all of the relevant metadata to the server. 

    // Let's check how many images we need to send.
    let amountOfArtToUpload = Object.values(characterImageFiles).filter((object) => object.state == "local").length;

    const messageToSend = {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify({
            step: "1",
            file_amount: amountOfArtToUpload
        })
    };

    console.log(`SENT: ${JSON.stringify(messageToSend)}`)

    updateErrorText(`Requesting permission to upload...`);

    const s3UrlsRequestResponse = await fetch(targetUrl, messageToSend);

    // ERROR! Bubble it up to user.
    if (s3UrlsRequestResponse.status >= 400 && s3UrlsRequestResponse.status < 600) {
        let errorText = await s3UrlsRequestResponse.text();
        updateErrorText(`<b>ERROR ${s3UrlsRequestResponse.status}, ${s3UrlsRequestResponse.statusText}:</b> ${errorText}`);
        return;
    }

    // A valid request should return a json with a list of "presigned_urls".

    let s3Urls = await s3UrlsRequestResponse.json();

    console.log(`RECIEVED: ${JSON.stringify(s3Urls)}`);

    // Alright, let's try uploading everything to S3.
    // Put everything in a list so we can run them in parallel later.
    let listOfUploadPromises = [];

    Object.entries(characterImageFiles).forEach(([key, value]) => {
        if (value.state == "local") {
            let targetUrl = s3Urls.presigned_urls.pop();
            listOfUploadPromises.push((async () => {
                await fetch(targetUrl, {
                    method: 'PUT',
                    body: value.file,
                    headers: {
                        'Content-Type': value.file.type
                    }
                })
                // TODO - check for errors.

                characterImageFiles[key].state = "uploaded";
                characterImageFiles[key].key = targetUrl;
            })());
        }
    });

    updateErrorText(`Uploading image files...`);

    await Promise.all(listOfUploadPromises);

    postInfo.thumbnail_key = characterImageFiles["thumbnail"].key;
    postInfo.page_img_key = characterImageFiles["pageImg"].key;

    if (characterImageFiles["logo"]) {
        postInfo.logo_url = characterImageFiles["logo"].key;
    }

    const finalMessageToSend = {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify({
            step: "2",
            ...postInfo
        })
    };

    console.log(`SENDING: ${JSON.stringify(finalMessageToSend)}`);

    updateErrorText(`Uploading character metadata...`);

    // Now that it's all on S3, send the final result!
    const finalUploadRequest = await fetch(targetUrl, finalMessageToSend);

    console.log(`UPLOAD COMPLETE! Result : ${JSON.stringify(finalUploadRequest)} `)

    // ERROR! Bubble it up to user.
    if (finalUploadRequest.status >= 400 && finalUploadRequest.status < 600) {
        let errorText = await finalUploadRequest.text();
        updateErrorText(`<b>ERROR ${finalUploadRequest.status}, ${finalUploadRequest.statusText}:</b> ${errorText}`);
        return;
    }
    // If there's a redirect, follow it, it means the upload was successful.
    else if (finalUploadRequest.redirected) {
        updateErrorText(`Upload successful!`);
        window.location.href = finalUploadRequest.url;
    }
}

// Sends a DELETE request to the given url. If no URL is passed, the current page.
async function sendDeleteRequest(targetUrl = window.location.pathname) {
    if (!confirm('Are you SURE you want to DELETE THIS POST? This CANNOT be undone!')) {
        return;
    }

    if (!confirm('Again, CANNOT BE UNDONE. Everything will be gone. Admins won\'t be able to restore it. You sure?')) {
        return;
    }

    await fetch(targetUrl, {
        method: 'DELETE'
    });
}