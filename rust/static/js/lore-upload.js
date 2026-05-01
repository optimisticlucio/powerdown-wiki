// Posts the current lore page info to the given target URL.
async function postLorePage(targetUrl = window.location.pathname) {

    let parentCategorySelect = document.getElementById("parentCategory");

    if (parentCategorySelect.options[parentCategorySelect.selectedIndex].disabled) {
        updateErrorText("You did not select a parent category!");
        return;
    }

    let parent_category_id = parseInt(parentCategorySelect.value);

    let slug = document.getElementById("pageSlug").value.trim();

    if (!slug) {
        updateErrorText("Page slug is missing.");
        return;
    }

    let title = document.getElementById("pageTitle").innerHTML.trim();

    if (!title) {
        updateErrorText("Title is missing.");
        return;
    }

    let content = document.getElementById("pageContents").innerHTML
        .replaceAll("<br>", "\n")
        .trim();

    let lorePageData = {
        step: "2",
        slug,
        title,
        content,
        parent_category_id,
    };

    let description = document.getElementById("pageDescription").value.trim();

    if (description) {
        lorePageData.description = description;
    }

    const messageToSend = {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify(lorePageData)
    };

    console.log(`SENT: ${JSON.stringify(messageToSend)}`)

    updateErrorText(`Sending lore page data...`);

    const categoryUpdateResponse = await fetch(targetUrl, messageToSend);

    if (categoryUpdateResponse.status >= 400 && categoryUpdateResponse.status < 600) {
        let errorText = await categoryUpdateResponse.text();
        updateErrorText(`<b>ERROR ${categoryUpdateResponse.status}, ${categoryUpdateResponse.statusText}:</b> ${errorText}`);
        return;
    }

    // If there's a redirect, follow it, it means the upload was successful.
    else if (categoryUpdateResponse.redirected) {
        updateErrorText(`Upload successful!`);
        window.location.href = categoryUpdateResponse.url;
    }
}


// For the lore category section - creates a new Category, with the given name and description
function createNewCategory(name = "New Category", description = null, id = null) {
    let categoryHolder = document.getElementsByClassName("categories")[0];

    let newCategory = document.createElement("div");
    newCategory.classList.add("category");
    if (id) {
        newCategory.dataset.id = id;
    }

    let categoryTitle = document.createElement("h3");
    categoryTitle.contentEditable = true;
    categoryTitle.innerHTML = name;

    let categoryDescription = document.createElement("div");
    categoryDescription.contentEditable = true;
    categoryDescription.classList.add("text");
    if (description) {
        categoryDescription.innerHTML = description;
    }

    newCategory.append(categoryTitle, categoryDescription);

    categoryHolder.appendChild(newCategory);
}

// Ran at when the edit-categories page is loaded to show the existing pages.
function generateGivenCategories(givenCategories) {
    givenCategories.forEach((givenCategory) => createNewCategory(givenCategory.name, givenCategory.description, givenCategory.id));
}

// For the Lore Categories page. Uploads all the current lore categories.
async function uploadLoreCategories(targetUrl = window.location.pathname) {

    // First lets get our data.
    let sendableCategoryData = [];
    let categoryHolder = document.getElementsByClassName("categories")[0];

    Array.from(categoryHolder.children).forEach((categoryDiv, index) => {
        let title = categoryDiv.querySelector("h3").innerHTML.trim();

        if (!title) {
            updateErrorText("One of the categories is missing a title!");
            return;
        }

        let order_position = index;

        let sendableCategory = {
            title,
            order_position
        };

        let description = categoryDiv.querySelector(".text").innerHTML.trim();

        if (description) {
            sendableCategory.description = description;
        }

        if (categoryDiv.dataset.id) {
            sendableCategory.id = parseInt(categoryDiv.dataset.id);
        }

        sendableCategoryData.push(sendableCategory);
    });

    const messageToSend = {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify(sendableCategoryData)
    };

    console.log(`SENT: ${JSON.stringify(messageToSend)}`)

    updateErrorText(`Sending new category data...`);

    const categoryUpdateResponse = await fetch(targetUrl, messageToSend);

    if (categoryUpdateResponse.status >= 400 && categoryUpdateResponse.status < 600) {
        let errorText = await categoryUpdateResponse.text();
        updateErrorText(`<b>ERROR ${categoryUpdateResponse.status}, ${categoryUpdateResponse.statusText}:</b> ${errorText}`);
        return;
    }

    window.location.reload();
}