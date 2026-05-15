const DEFAULT_SRC = '/static/img/pd_logo_with_stroke.png';

document.querySelectorAll('.thumbnail-wrap').forEach(applyRelevantActions);

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
    image.classList.add("thumb-img");
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