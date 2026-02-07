// Tags is [], unless there's already .tag-name on initialization, at which point it'll be those tags.
const tags = Array.from(document.querySelectorAll('.tag-name')).map(el => el.innerHTML);

// Assumed to exist:
// <div id="tag-search" data-base-url="{some target URL goes here}"> <input type="text" id="tag-input" autocomplete="off" /> </div>
const tagInput = document.getElementById('tag-input');
const tagsContainer = document.getElementById('tag-search');

// We are assuming that the tags container is telling us what the target URL is. If not, well shit.
const baseURL = tagsContainer.dataset.baseUrl;

const tagRemoveOnClick = (event) => {
    const self = event.currentTarget; 
    const idx = tags.indexOf(self.parentElement.querySelector(".tag-name").innerHTML);
    if (idx > -1) {
        tags.splice(idx, 1);
        self.parentElement.remove();
    }
    window.location.href = createTargetUrl();
};

document.querySelectorAll('.tag-remove').forEach( (tagRemove) => {tagRemove.addEventListener('click', tagRemoveOnClick)});

function createTag(text) {
    const tag = document.createElement('div');
    tag.className = 'tag';
    tag.innerHTML = `
        <span class="tag-name">${text}</span>
        <span class="tag-remove">×</span>
    `;
    
    tag.querySelector('.tag-remove').addEventListener('click', tagRemoveOnClick);
    
    tagsContainer.insertBefore(tag, tagInput);
}

function addTag(text) {
    text = text.trim().replace(/[,\s]/g, '');
    
    if (text && !tags.includes(text)) {
        tags.push(text);
        createTag(text);
    }
    
    tagInput.value = '';
}

function createTargetUrl() {
    // We start with the current URL's parameter and remove whatever we don't need or want.
    let targetParameters = new URLSearchParams(window.location.search);

    targetParameters.delete("page");
    targetParameters.delete("tags");

    if (tags.length > 0) {
        targetParameters.set("tags", tags.join(","));
    }

    if (targetParameters.size > 0) {
        return `${baseURL}?${targetParameters.toString()}`;
    } else {
        return baseURL;
    }
}

tagInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
        e.preventDefault();
        if (tagInput.value.trim()) {
            addTag(tagInput.value);
            window.location.href = createTargetUrl();
        } 
    } else if (e.key === 'Backspace' && tagInput.value === '' && tags.length > 0) {
        const tagElements = tagsContainer.querySelectorAll('.tag');
        const lastTagElement = tagElements[tagElements.length - 1];
        
        tags.pop();
        lastTagElement.remove();
        window.location.href = createTargetUrl();
    }
});