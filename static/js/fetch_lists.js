async function fetch_lists() {
  const response = await fetch("api/lists")
  const lists = await response.json()
  return lists
}

async function main() {
  const lists_parent = document.getElementById("lists")
  const lists = await fetch_lists()
  lists.forEach((list) => {
    console.log(list)
    const element = document.createElement("li")
    const name_element = document.createElement("a")
    name_element.innerText = list.name
    name_element.href = `list/${list.id}`

    const users_list = document.createElement("ul")
    if (list.users) {
      list.users.forEach((user) => {
        const user_element = document.createElement("li")
        user_element.innerText = user
        users_list.appendChild(user_element)
      })
    }

    element.appendChild(name_element)
    element.appendChild(users_list)

    lists_parent.appendChild(element)
  })
}

main()
