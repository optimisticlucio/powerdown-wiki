// @tscheck 


const imageContainer = document.getElementById("postImages");

// An array holding the files that are in the image container.
let filesInImageContainer = []; 

// Incase someone exists and enters the page - clear out the image container, just to be sure.
imageContainer.innerHTML = '';


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

  const postImages = [...filesInImageContainer];
  const postThumbnail = document.getElementById("postThumbnail").files[0];

  // Most values are checked by the server, but thumbnail is not checked before being sent to s3. So let's make sure it's set.
  if (!postThumbnail) {
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

  // For some reason, invalid dates gives a very weird serde error, so let's notify the user 

  // We have all of our data, sexcellent.
  // Posting needs to be done in two phases - we ask for S3 presigned URLs to upload our images to,
  // and after that, we send all of the relevant metadata to the server. 

  const messageToSend = {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    credentials: "same-origin",
    body: JSON.stringify({
      step: "1",
      art_amount: postImages.length
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

  // A valid request should return a json with "thumbnail_presigned_url" which is one url, 
  // and "art_presigned_urls" which is a list.

  let s3Urls = await s3UrlsRequestResponse.json();

  console.log(`RECIEVED: ${JSON.stringify(s3Urls)}`);

  // TODO: Remove this conversion once we go live, it's only here bc we're working with localStack instead of a live server.
  s3Urls.thumbnail_presigned_url = s3Urls.thumbnail_presigned_url.replace("host.docker.internal", "localhost.localstack.cloud");
  s3Urls.art_presigned_urls = s3Urls.art_presigned_urls.map((presigned_url) => presigned_url.replace("host.docker.internal", "localhost.localstack.cloud"));

  postInfo.thumbnail_key = s3Urls.thumbnail_presigned_url;
  postInfo.art_keys = s3Urls.art_presigned_urls;


  // Alright, let's try uploading everything to S3.

  const thumbnailURLattempt = fetch(s3Urls.thumbnail_presigned_url, {
    method: 'PUT',
    body: postThumbnail,
    headers: {
      'Content-Type': postThumbnail.type
    }
  });

  const artUploadAttempts = s3Urls.art_presigned_urls.map(
    (presignedUrl, index) => fetch(presignedUrl, {
      method: 'PUT',
      body: postImages[index],
      headers: {
        'Content-Type': postImages[index].type
      }
    })
  )

  await Promise.all([thumbnailURLattempt, ...artUploadAttempts]);

  //TODO - Check for Errors.

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

// Ran when the user selects a new file to be added to the image section.
function addNewImage(event) {
  const file = event.target.files[0];

  // We assume it's one of the allowed filetypes. If you broke it, not my fuckin problem, this is the client.

  if (file) {
    // Put it in the list
    filesInImageContainer.push(file);

    // Now, make the user see the new file listed.
    const reader = new FileReader();
    
    reader.onload = (e) => {
      // Create an img element
      const img = document.createElement('img');
      img.src = e.target.result; // This is the base64 data URL
      img.style.maxWidth = '500px'; 
      
      // Add it to the page
      imageContainer.appendChild(img);
    };
    
    reader.readAsDataURL(file); // Read as data URL for images
    event.target.value = '';
  }
}
