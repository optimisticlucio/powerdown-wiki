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

        // TODO: Get Thumbnail

        let data_to_send_back = {
            title,
            description,
            url,
            order_position: parseInt(index)
        };

        if (id) {
            data_to_send_back.id = parseInt(id);
        }

        return data_to_send_back;
    });

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