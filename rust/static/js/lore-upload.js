// Checks the page validity of a new lore page upload. If valid, sends new art to the site.
async function attemptNewLorePageUpload(targetUrl = window.location.pathname) {
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