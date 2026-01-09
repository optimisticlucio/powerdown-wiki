const imageContainer = document.getElementById("postImages");

// Checks the page validity of /art/new. If valid, sends new art to the site.
function attemptNewArtUpload() {

}

// Ran when the user selects a new file to be added to the image section.
function addNewImage(event) {
    const file = event.target.files[0];

    // TODO: Make sure it is one of the allowed filetypes.

    if (file) {
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