function filterModules() {
  const input = document.getElementById('search');
  const filter = input.value.toLowerCase();
  const list = document.getElementById('module-list');
  const items = list.getElementsByTagName('li');

  for (let i = 0; i < items.length; i++) {
    const a = items[i].getElementsByTagName('a')[0];
    const text = a.textContent || a.innerText;
    if (text.toLowerCase().indexOf(filter) > -1) {
      items[i].style.display = "";
    } else {
      items[i].style.display = "none";
    }
  }
}

// Add smooth scroll for anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        document.querySelector(this.getAttribute('href')).scrollIntoView({
            behavior: 'smooth'
        });
    });
});
