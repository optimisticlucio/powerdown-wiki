// Tags is [], unless there's already .tag-name on initialization, at which point it'll be those tags.
const tags = Array.from(document.querySelectorAll('.tag-name')).map(el => el.innerHTML);
const tagInput = document.getElementById('tag-input');
const tagsContainer = document.getElementById('tag-search');
const baseURL = tagsContainer.dataset.baseUrl;

const tagRemoveOnClick = (event) => {
    const self = event.currentTarget; // or event.target, depending on your HTML structure
    const idx = tags.indexOf(self.parentElement.querySelector(".tag-name").innerHTML);
    if (idx > -1) {
        tags.splice(idx, 1);
        self.parentElement.remove();
    }
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

tagInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
        e.preventDefault();
        if (tagInput.value.trim()) {
            addTag(tagInput.value);
        } else if (tags.length > 0) {
            const url = `${baseURL}?tags=${tags.join(',')}`;
            window.location.href = url;
        } else {
            window.location.href = baseURL;
        }
    } else if (e.key === 'Backspace' && tagInput.value === '' && tags.length > 0) {
        const lastTag = tags[tags.length - 1];
        const tagElements = tagsContainer.querySelectorAll('.tag');
        const lastTagElement = tagElements[tagElements.length - 1];
        
        tags.pop();
        lastTagElement.remove();
    }
});