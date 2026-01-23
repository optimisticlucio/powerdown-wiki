// @tscheck 

// Image files here are represented with an object, bc we need to discriminate between images
// already uploaded to the DB and those that are not. they all have the "state" property,
// which can be "uploaded" or "local". "local" means there's another property named "file"
// pointing to the local file that needs to be updated. "uploaded" means there's another
// property called "key" pointing to the image's current URL.

const imageContainer = document.getElementById("postImages");
const thumbnailContainer = document.getElementById("postThumbnail");

// Initialize only if not already defined in the page
if (typeof filesInImageContainer === 'undefined') {
  // An array holding the files that are in the image container.
    filesInImageContainer = [];
}
if (typeof thumbnailObject === 'undefined') {
    thumbnailObject = {};
}

// Incase someone exits and enters the page, to ensure there aren't visual/logic discrepancies - clear out the image and thumbnail containers.
imageContainer.innerHTML = '';
thumbnailContainer.innerHTML = '';

// Now we render whatever is in the files and object variables.
filesInImageContainer.forEach( (givenImage) => {
  const img = document.createElement('img');
  img.src = givenImage.key;
  imageContainer.appendChild(img);
})
// If thumbnail object isn't empty.
if (Object.keys(thumbnailObject).length > 0) {
  const img = document.createElement('img');
  img.src = thumbnailObject.key;
  thumbnailContainer.appendChild(img);
}

// Checks the page validity of /art/new. If valid, sends new art to the site.
async function attemptNewArtUpload(targetUrl = window.location.pathname) {
  // First, let's see if all the values are valid on our end.

  // Get all the inputs under the wrapper, and check their validity.
  const inputsToCheck = Array.from(document.querySelectorAll(".upload input"));
  if (inputsToCheck.some((inputItem) => !inputItem.checkValidity())) {
    document.getElementById("errorDisplay").innerHTML = `<b>ERROR:</b> Some of the values are either not set or invalid. Fix all the sections that are highlighted in red!`;
  }

  // Now, let's collect all of our data.
  const postTitle = document.getElementById("postTitle").value;

  let postInfo = {
    title: postTitle,
    creation_date: document.getElementById("postCreationDate").value,
    is_nsfw: document.getElementById("postIsNsfw").checked,
    creators: document.getElementById("postArtists").value.split(","),
    slug: document.getElementById("postSlug").value || postTitle.toLowerCase().replaceAll(" ","-"),
  };

  // Most values are checked by the server, but thumbnail is not checked before being sent to s3. So let's make sure it's set.
  if (!thumbnailObject) {
    document.getElementById("errorDisplay").innerHTML = `<b>ERROR:</b> Thumbnail wasn't selected.`;
  }

  // Now add optional values
  const description = document.getElementById("postDescription").value;
  if (description) {
    // If not empty/not falsy
    postInfo.description = description;
  }

  // Remove all empty tags.
  const tags = document.getElementById("postTags").value.split(",").filter((tag) => tag);
  if (tags.length > 0) {
    postInfo.tags = tags;
  }

  // TODO: Check for valid calendar date and throw error if invalid bc server returns a weird serde if that's missing.

  // We have all of our data, sexcellent.
  // Posting needs to be done in two phases - we ask for S3 presigned URLs to upload our images to,
  // and after that, we send all of the relevant metadata to the server. 

  // Let's check how many images we need to send.
  let amountOfArtToUpload = filesInImageContainer.filter((object) => object.state == "local").length + (thumbnailObject.state == "local" ? 1 : 0);

  const messageToSend = {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    credentials: "same-origin",
    body: JSON.stringify({
      step: "1",
      art_amount: amountOfArtToUpload
    })
  };

  console.log(`SENT: ${JSON.stringify(messageToSend)}`)

  const s3UrlsRequestResponse = await fetch(targetUrl, messageToSend);

  // ERROR! Bubble it up to user.
  if (s3UrlsRequestResponse.status >= 400 && s3UrlsRequestResponse.status < 600) {
    let errorText = await s3UrlsRequestResponse.text();
    document.getElementById("errorDisplay").innerHTML = `<b>ERROR ${s3UrlsRequestResponse.status}, ${s3UrlsRequestResponse.statusText}:</b> ${errorText}`;
    return;
  }

  // A valid request should return a json with a list of "presigned_urls".

  let s3Urls = await s3UrlsRequestResponse.json();

  console.log(`RECIEVED: ${JSON.stringify(s3Urls)}`);

  // TODO: Remove this conversion once we go live, it's only here bc we're working with localStack instead of a live server.
  s3Urls.presigned_urls = s3Urls.presigned_urls.map((presigned_url) => presigned_url.replace("host.docker.internal", "localhost.localstack.cloud"));

  // Alright, let's try uploading everything to S3.
  // Put everything in a list so we can run them in parallel later.
  let listOfUploadFunctions = [];

  if (thumbnailObject.state == "local") {
    // Upload thumbnail to server, then reassign the appropriate values.
    let thumbnailKey = s3Urls.presigned_urls.pop();
    listOfUploadFunctions.push((async () => {
      await fetch(thumbnailKey, {
        method: 'PUT',
        body: thumbnailObject.file,
        headers: {
          'Content-Type': thumbnailObject.file.type
        }
      })
      // TODO - check for errors.

      thumbnailObject.state = "uploaded";
      thumbnailObject.key = thumbnailKey;
    })()); 
  }

  listOfUploadFunctions.push(...filesInImageContainer
  .filter((imageObject) => imageObject.state == "local")
  .map((imageObject, index) => {
    const urlToUpload = s3Urls.presigned_urls[index];
    
    return (async () => {  
      await fetch(urlToUpload, {
        method: 'PUT',
        body: imageObject.file,
        headers: {
          'Content-Type': imageObject.file.type
        }
      });

      // TODO: Check for errors

      imageObject.state = "uploaded";
      imageObject.key = urlToUpload;
    })();
  }
));

  await Promise.all(listOfUploadFunctions);

  postInfo.thumbnail_key = thumbnailObject.key;
  postInfo.art_keys = filesInImageContainer.map((imageObject) => imageObject.key);

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

  // Now that it's all on S3, send the final result!
  const finalUploadRequest = await fetch(targetUrl, finalMessageToSend);

  // TODO: Show to the user the response. In the meanwhile, the console will do.
  console.log(`UPLOAD COMPLETE! Result : ${JSON.stringify(finalUploadRequest)} `)

  // ERROR! Bubble it up to user.
  if (finalUploadRequest.status >= 400 && finalUploadRequest.status < 600) {
    let errorText = await finalUploadRequest.text();
    document.getElementById("errorDisplay").innerHTML = `<b>ERROR ${finalUploadRequest.status}, ${finalUploadRequest.statusText}:</b> ${errorText}`;
    return;
  }
  // If there's a redirect, follow it, it means the upload was successful.
  else if (finalUploadRequest.redirected) {
    window.location.href = finalUploadRequest.url;
  }
}

// Sends a DELETE request to the given url. If no URL is passed, the current page.
async function sendDeleteRequest(targetUrl = window.location.pathname) {
  await fetch(targetUrl, {
    method: 'DELETE'
  });
}

// Ran when the user selects a new file to be added to the image section.
function addNewImage(event) {
  const file = event.target.files[0];

  // We assume it's one of the allowed filetypes. If you broke it, not my fuckin problem, this is the client.

  if (file) {
    // Put it in the list
    filesInImageContainer.push({
      state: "local",
      file: file
    });

    // Now, make the user see the new file listed.
    const reader = new FileReader();
    
    reader.onload = (e) => {
      // Create an img element
      const img = document.createElement('img');
      img.src = e.target.result; // This is the base64 data URL
      
      // Add it to the page
      imageContainer.appendChild(img);
    };
    
    reader.readAsDataURL(file); // Read as data URL for images
    event.target.value = '';
  }
}

// Ran when the user selects a thumbnail.
function setThumbnail(event) {
  const file = event.target.files[0];

  // We assume it's one of the allowed filetypes. If you broke it, not my fuckin problem, this is the client.

  if (file) {
    // Whatever was the old thumbnail doesn't matter anymore.
    thumbnailObject = {
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
      thumbnailContainer.innerHTML='';
      // Add it to the page
      thumbnailContainer.appendChild(img);
    };
    
    reader.readAsDataURL(file); // Read as data URL for images
    event.target.value = '';
  }
}
