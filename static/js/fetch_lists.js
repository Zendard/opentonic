const user = document.getElementById("user").innerText
const url_pfx = document.getElementById("url_pfx").innerText

async function fetch_lists() {
  const response = await fetch(`${url_pfx}/api/lists`)
  const lists = await response.json()
  return lists
}

async function main() {
  const lists_parent = document.getElementById("lists")
  const lists = await fetch_lists()
  lists.forEach((list) => {
    const clone = document.importNode(document.getElementById("list_template").content, true)

    const name_element = clone.querySelector("a")
    name_element.innerText = list.name
    name_element.href = `${url_pfx}/list/${list.id}`

    const owner_element = clone.querySelector("p")
    owner_element.innerText = list.owner

    lists_parent.appendChild(clone)
  })
}

main()
