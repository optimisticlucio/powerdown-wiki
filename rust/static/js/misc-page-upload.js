const DEFAULT_SRC = 'https://placehold.co/400x260/e8e4de/999?text=No+thumbnail';

document.querySelectorAll('.thumbnail-wrap').forEach(applyRelevantActions);

function applyRelevantActions(wrap) {
    const img = wrap.querySelector('.thumb-img');
    const input = wrap.querySelector('input[type="file"]');
    const removeBtn = wrap.querySelector('.remove-btn');

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

    // TODO: LEFT

    // TODO: RIGHT

    miscHolder.appendChild(miscItemElement);
}