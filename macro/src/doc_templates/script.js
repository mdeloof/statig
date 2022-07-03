function renderDocs() {
    let body = document.querySelector("body");
    let bodyStyle = getComputedStyle(body);
    let stateMachine = document.querySelector(".state-machine");

    let backgroundColor = bodyStyle.backgroundColor;

    let state_names = [];

    document.querySelectorAll(".state").forEach(elem => {
        let state = elem.parentElement;
        let state_name = elem.querySelector(".state-name");
        let state_action = elem.querySelector(".state-action");
        let state_content = elem.querySelector(".state-content");

        let method_section = document.getElementById(`method.${state_name.textContent}`);
        let method_summary = method_section.parentElement;

        if (method_summary.tagName == "SUMMARY") {
            method_docblock = method_summary.nextElementSibling;
            state_content.insertBefore(method_docblock.cloneNode(true), state_content.firstChild);
        }

        state_names.push(state_name);

        state_name.style.backgroundColor = backgroundColor;
    });

    console.log(state_names);

    state_names.forEach(state => {
        stateMachine.querySelectorAll(`a[href$='#method.${state.textContent}']`).forEach(elem => {
            window.scrollY;
            let level;
            for (let c of state.classList.values()) {
                let result = c.match(/level-(\d)/);
                if (result != null) {
                    level = parseInt(result[1])
                }
            }

            // Don't do anything to URL path.
            elem.href = "javascript:void(0);";

            // Scroll to state.
            elem.onclick = function () {
                let offset = 40 * (level - 1);
                let scroll_position = state.parentElement.getBoundingClientRect().top + window.scrollY - offset;
                window.scrollTo({
                    top: scroll_position,
                    behavior: "smooth"
                });
            }
        });
    })
}

window.onload = renderDocs;