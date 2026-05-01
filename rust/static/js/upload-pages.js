/// Given text, puts it in the Error Text div.
function updateErrorText(text) {
    document.getElementById("errorDisplay").innerHTML = text;
}

/// Properly returns the contents of a contenteditable div.
function getContentEditableText(el) {
    let clone = el.cloneNode(true);

    clone.querySelectorAll('br').forEach(br => br.replaceWith('\n'));
    clone.querySelectorAll('div, p').forEach(block => {
        block.prepend('\n');
        block.replaceWith(...Array.from(block.childNodes));
    });

    return clone.textContent.trim();
}