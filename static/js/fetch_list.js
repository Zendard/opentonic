const user = document.getElementById("user").innerText
const url_pfx = document.getElementById("url_pfx").innerText
const list_id = document.getElementById("list_id").innerText

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

main()
