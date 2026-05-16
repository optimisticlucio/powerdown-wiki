const DEFAULT_SRC = '/static/img/pd_logo.svg';

function applyRelevantActions(wrap) {
    const img = wrap.querySelector('.thumb-img');
    const input = wrap.querySelector('input[type="file"]');
    const removeBtn = wrap.querySelector('.removeButton');

    function showRemove(visible) {
        removeBtn.style.display = visible ? 'flex' : 'none';
    }

    if (img.dataset.hasThumb === 'true') showRemove(true);

    input.addEventListener('change', () => {
        const file = input.files[0];
        if (!file) return;
        img.src = URL.createObjectURL(file);
        img.dataset.hasThumb = 'true';
        showRemove(true);
    });

    removeBtn.addEventListener('click', e => {
        e.stopPropagation();
        img.src = DEFAULT_SRC;
        img.dataset.hasThumb = 'false';
        input.value = '';
        showRemove(false);
    });
}

function createNewMiscItem() {
    const miscHolder = document.querySelector('.misc-holder');

    const miscItemElement = document.createElement("div");
    miscItemElement.classList.add("misc");
    miscItemElement.draggable = true;

    const leftElement = document.createElement("div");
    leftElement.classList.add("left");

    const miscTitle = document.createElement("h1");
    miscTitle.contentEditable = true;
    miscTitle.innerHTML = "Write Title Here";

    const miscDescription = document.createElement("h2");
    miscDescription.contentEditable = true;
    miscDescription.innerHTML = "Write description here.";

    const linksWrapper = document.createElement("p");
    linksWrapper.innerHTML = "Links to:";

    const linksTo = document.createElement("span");
    linksTo.contentEditable = true;
    linksTo.innerHTML = "Insert link here";
    linksWrapper.appendChild(linksTo);

    leftElement.append(miscTitle, miscDescription, document.createElement("hr"), linksWrapper);

    const thumbnailElement = document.createElement("div");
    thumbnailElement.classList.add("thumbnail", "thumbnail-wrap");

    const image = document.createElement("img");
    image.classList.add("thumb-img", "thumbnail");
    image.dataset.hasThumb = false;
    image.src = DEFAULT_SRC;

    const invisibleInput = document.createElement("input");
    invisibleInput.type = "file";
    invisibleInput.accept = "image/*";
    invisibleInput.classList.add("hiddenThumbnailInput");

    const deleteButton = document.createElement("button");
    deleteButton.type = 'button';
    deleteButton.classList.add("removeButton", "dark");
    deleteButton.innerHTML = "&#x2715;";

    thumbnailElement.append(image, invisibleInput, deleteButton);

    miscItemElement.append(leftElement, thumbnailElement);

    applyRelevantActions(miscItemElement);

    miscHolder.appendChild(miscItemElement);
}

async function updateMiscItems(targetUrl = window.location.pathname) {
    const miscItemElements = document.querySelectorAll(".misc-holder > .misc");

    const miscItems = [...miscItemElements].map((miscItemElement, index) => {
        let title = getContentEditableText(miscItemElement.querySelector("h1"));
        let description = getContentEditableText(miscItemElement.querySelector("h2"));
        let url = getContentEditableText(miscItemElement.querySelector("span"));

        let id = miscItemElement.id;

        let image_element = miscItemElement.querySelector(".thumb-img");

        let data_to_send_back = {
            title,
            description,
            url,
            order_position: parseInt(index)
        };

        if (id) {
            data_to_send_back.id = parseInt(id);
        }

        if (image_element.dataset.hasThumb == 'true') {
            data_to_send_back.thumbnail_url = image_element.src;
        }

        return data_to_send_back;
    });

    // We have all the data organized. Do we have any images to send?

    let miscItemsWithNewThumbnails = miscItems.filter((miscItem) => {
        return miscItem.thumbnail_url?.startsWith("blob:");
    });

    if (miscItemsWithNewThumbnails.length > 0) {
        // Yep, let's get on it.
        const messageToSend = {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: "same-origin",
            body: JSON.stringify({
                step: "1",
                file_amount: miscItemsWithNewThumbnails.length
            })
        };

        console.log(`SENT: ${JSON.stringify(messageToSend)}`)

        updateErrorText(`Requesting permission to upload thumbnails...`);

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
        let listOfUploadFunctions = miscItemsWithNewThumbnails.map((miscItem, index) => {
            const urlToUpload = s3Urls.presigned_urls[index];

            return (async () => {
                let imageFile = await fetch(miscItem.thumbnail_url).then(r => r.blob());

                await fetch(urlToUpload, {
                    method: 'PUT',
                    body: imageFile,
                    headers: {
                        'Content-Type': imageFile.type
                    }
                });

                miscItem.thumbnail_url = urlToUpload;
            })();
        }
        );

        updateErrorText(`Uploading thumbnails...`);

        await Promise.all(listOfUploadFunctions);
    }

    const messageToSend = {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify({
            step: "2",
            misc_items: miscItems
        })
    };

    console.log(`SENT: ${JSON.stringify(messageToSend)}`)

    updateErrorText(`Sending misc list data...`);

    const categoryUpdateResponse = await fetch(targetUrl, messageToSend);

    if (categoryUpdateResponse.status >= 400 && categoryUpdateResponse.status < 600) {
        let errorText = await categoryUpdateResponse.text();
        updateErrorText(`<b>ERROR ${categoryUpdateResponse.status}, ${categoryUpdateResponse.statusText}:</b> ${errorText}`);
        return;
    }

    updateErrorText(`Upload successful!`);
    window.location.href = categoryUpdateResponse.url;
}

document.querySelectorAll('.thumbnail-wrap').forEach(applyRelevantActions);

const container = document.getElementsByClassName('misc-holder')[0];
let draggedEl = null;

container.addEventListener('dragstart', e => {
    draggedEl = e.target.closest('.misc');
    e.target.style.opacity = '0.4';
});

container.addEventListener('dragend', e => {
    e.target.style.opacity = '';
});

container.addEventListener('dragover', e => {
    e.preventDefault(); // required to allow dropping
    const target = e.target.closest('.misc');
    if (target && target !== draggedEl) {
        // Insert before or after based on mouse position
        const rect = target.getBoundingClientRect();
        const after = e.clientY > rect.top + rect.height / 2;
        container.insertBefore(draggedEl, after ? target.nextSibling : target);
    }
});