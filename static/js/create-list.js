const user = document.getElementById("user").innerText
const url_pfx = document.getElementById("url_pfx").innerText

const form = document.getElementById("form")
form.addEventListener("submit", submitForm)

async function submitForm(_) {
  const form_data = new FormData(form)
  const res = await fetch(`${url_pfx}/api/create-list`, {
    method: "POST",
    body: JSON.stringify(Object.fromEntries(form_data)),
    headers: { "Content-Type": "application/json" }
  })
  // Redirect to index if success
  if (res.status == 200) {
    window.location.href = `${url_pfx}`
  }
}
