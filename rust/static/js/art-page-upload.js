// @tscheck 
const imageContainer = document.getElementById("postImages");
let filesInImageContainer = []; // An array holding the files that are in the image container.

// Incase someone exists and enters the page - clear out the image container, just to be sure.
imageContainer.innerHTML = '';


// Checks the page validity of /art/new. If valid, sends new art to the site.
async function attemptNewArtUpload() {
  // First, let's collect all of our data.
  const postTitle = document.getElementById("postTitle").value;

  let postInfo = {
    title: postTitle,
    creation_date: document.getElementById("postCreationDate").value,
    is_nsfw: document.getElementById("postIsNsfw").checked,
    creators: document.getElementById("postArtists").value.split(","),
    slug: document.getElementById("postSlug").value || postTitle.toLowerCase().replaceAll(" ","-"),
    tags: document.getElementById("postTags").value.split(","),
  };

  const postImages = [...filesInImageContainer];
  const postThumbnail = document.getElementById("postThumbnail").value

  // We have all of our data, sexcellent.
  // Posting needs to be done in two phases - we ask for S3 presigned URLs to upload our images to,
  // and after that, we send all of the relevant metadata to the server. 

  const s3UrlsRequestResponse = await fetch("/art/new", {
    method: "POST",
    body: JSON.stringify({
      step: 1,
      art_amount: postImages.length
    })
  });

  // TODO - Handle an error here.

  // A valid request should return a json with "thumbnail_presigned_url" which is one url, 
  // and "art_presigned_urls" which is a list.

  const s3Urls = JSON.parse(s3UrlsRequestResponse);

  const thumbnailURLattempt = await fetch(s3Urls.thumbnail_presigned_url, {
    method: 'PUT',
    body: postThumbnail,
    headers: {
      'Content-Type': postThumbnail.type
    }
  });

  postInfo.thumbnail_key = s3Urls.thumbnail_presigned_url;

  // TODO - Send other files to S3, add their S3 URLs to postInfo.

  // Now that it's all on S3, send the final result!
  const finalUploadRequest = await fetch("/art/new", {
    method: "POST",
    body: JSON.stringify({
      step: 2,
      ...postInfo
    })
  });

  // TODO: Show to the user the response. In the meanwhile, the console will do.
  console.log(`UPLOAD COMPLETE! Result : ${JSON.stringify(finalUploadRequest)} `)
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
