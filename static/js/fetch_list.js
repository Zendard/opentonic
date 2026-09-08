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
  list.list_items.forEach((list_item) => {
    const list_item_element = document.createElement("li")
    const list_item_name = document.createElement("h4")
    list_item_name.innerText = list_item.name
    list_item_element.appendChild(list_item_name)
    const list_item_subtext = document.createElement("p")
    list_item_subtext.innerText = `Added by ${list_item.added_by} on ${list_item.added_on}`
    list_item_element.appendChild(list_item_subtext)
    list_items_list.appendChild(list_item_element)
  })
}

async function addListItem(_) {
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

add_list_item_form.addEventListener("submit", addListItem)
main()


