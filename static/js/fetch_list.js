const user = document.getElementById("user").innerText
const url_pfx = document.getElementById("url_pfx").innerText
const list_id = document.getElementById("list_id").innerText
const add_list_item_form = document.getElementById("add_list_item")

async function fetch_list() {
  const response = await fetch(`${url_pfx}/api/list/${list_id}`)
  const lists = await response.json()
  return lists
}

async function main() {
  const list = await fetch_list()

  document.getElementById("list_name").innerText = list.name
  const users_list = document.getElementById("users")
  list.users.forEach((user) => {
    const user_element = document.createElement("li")
    user_element.innerText = user
    users_list.appendChild(user_element)
  })

  append_list_items(list)
}
async function append_list_items(list) {
  const list_items_list = document.getElementById("list_items")
  const now = new Date(Date.now())
  list.list_items.forEach((list_item) => {
    const list_item_element = document.createElement("li")
    const div = document.createElement("div")
    const list_item_name = document.createElement("h4")
    list_item_name.innerText = list_item.name
    div.appendChild(list_item_name)
    const list_item_subtext = document.createElement("p")
    const added_on = new Date(Date.parse(list_item.added_on))
    if (added_on.toDateString() == now.toDateString()) {
      list_item_subtext.innerText = `Added by ${list_item.added_by} at ${new String(added_on.getHours()).padStart(2, 0)}:${new String(added_on.getMinutes()).padStart(2, 0)}`
    } else {
      list_item_subtext.innerText = `Added by ${list_item.added_by} on ${added_on.getDay()}/${added_on.getMonth()}`
    }
    div.appendChild(list_item_subtext)
    const list_item_checkbox = document.createElement("button")
    list_item_checkbox.dataset["list_item_id"] = list_item.id
    list_item_checkbox.classList.add("check-button")
    list_item_checkbox.classList.add("hover")
    if (list_item.checked) {
      list_item_checkbox.classList.add("checked")
    }
    list_item_checkbox.addEventListener("click", toggle_list_item_check)
    list_item_element.appendChild(div)
    list_item_element.appendChild(list_item_checkbox)
    list_items_list.appendChild(list_item_element)
  })
}

async function toggle_list_item_check(e) {
  const button = e.target
  const list_item_id = button.dataset["list_item_id"]
  button.classList.add("loading")
  const checked = !button.classList.contains("checked")
  const response = await fetch(`${url_pfx}/api/check-list-item/${list_item_id}?checked=${checked}`, { method: "POST" })
  if (response.ok) {
    button.classList.toggle("checked")
    button.classList.remove("loading")
  }
}

async function add_list_item(_) {
  const form_data = new FormData(add_list_item_form)
  const res = await fetch(`${url_pfx}/api/add-list-item/${list_id}`, {
    method: "POST",
    body: JSON.stringify(Object.fromEntries(form_data)),
    headers: { "Content-Type": "application/json" }
  })

  if (res.ok) {
    const list_items_list = document.getElementById("list_items")
    list_items_list.textContent = ""

    const list = await fetch_list()
    append_list_items(list)
    add_list_item_form.reset()
  }
}

add_list_item_form.addEventListener("submit", add_list_item)
main()

