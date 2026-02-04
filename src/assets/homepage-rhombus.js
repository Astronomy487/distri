let rhombusColors = ["cyan", "yellow", "magenta"];
{
	function isThanksgiving(a=new Date){const b=a.getFullYear(),c=10,d=new Date(b,c,1),e=d.getDay();return a.getMonth()===c&&a.getDate()===1+(4-e+7)%7+21}
	let theDate = new Date().getMonth()+1+"-"+new Date().getDate();
	if (theDate == "3-17") rhombusColors = ["#169B62", "#FF883E", "#FFFFFF"];
	if (theDate == "3-31") rhombusColors = ["#3BBEFA", "#F589D8", "#FFFFFF"];
	if (theDate == "7-4") rhombusColors = ["#FFFFFF", "#224CE2", "#E52F29"];
	if (theDate == "10-31") rhombusColors = ["#E36622", "#666666", "#9315C1"];
	if (theDate == "12-25") rhombusColors = ["#FFFFFF", "#46903A", "#D93344"];
	if (isThanksgiving()) rhombusColors = ["#E67E22", "#76471B", "#F1C40F"];
	if (theDate == "2-14") rhombusColors = ["#E53935", "#F48FB1", "#FFFFFF"];
}

const TRI_WIDTH = 50;
const TRI_HEIGHT = Math.floor(TRI_WIDTH * Math.sqrt(3) / 2);

const RHOMBUS_TEMPLATES = [
	[0, 0, 0, TRI_WIDTH, TRI_HEIGHT * 2, "0% 50%, 50% 0%, 100% 50%, 50% 100%"],
	[1, -TRI_WIDTH, 0, TRI_WIDTH * 1.5, TRI_HEIGHT, "0% 100%, 33.3% 0%, 100% 0%, 66.7% 100%"],
	[2, -TRI_WIDTH, TRI_HEIGHT, TRI_WIDTH * 1.5, TRI_HEIGHT, "0% 0%, 33.3% 100%, 100% 100%, 66.7% 0%"]
];

const rhombusContainer = document.querySelector("rhombus-container");
let rhombusSets = [new Set(), new Set()];
let createdCells = new Set();
let swapTimer = null;

function cellKey(x, y) {
	return `${x},${y}`;
}

function createRhombus(template, baseX, baseY) {
	const [colorIndex, addX, addY, width, height, clipPath] = template;

	const el = document.createElement("a-rhombus");
	el.style.width = `${width}px`;
	el.style.height = `${height}px`;
	el.style.left = `${baseX + addX}px`;
	el.style.top = `${baseY + addY - 32*16}px`;
	el.style.background = rhombusColors[colorIndex];
	el.style.clipPath = `polygon(${clipPath})`;
	el.innerHTML = "&nbsp;";

	const state = Math.random() < 0.1 ? 1 : 0;
	el.dataset.state = state;
	el.style.opacity = state;

	rhombusSets[state].add(el);

	el.addEventListener("click", () => {
		const s = Number(el.dataset.state);
		swapRhombus(s, el);
	});

	rhombusContainer.appendChild(el);
}

function ensureGrid() {
	const ww = window.innerWidth;
	const hh = window.innerHeight;

	const startX = -TRI_WIDTH * 2;
	const startY = -TRI_HEIGHT * 2;

	let oddEven = true;

	for (let x = startX; x < ww + TRI_WIDTH * 2; x += TRI_WIDTH * 1.5) {
		oddEven = !oddEven;
		const yStart = startY - (oddEven ? 0 : TRI_HEIGHT);

		for (let y = yStart; y < hh + TRI_HEIGHT * 2; y += TRI_HEIGHT * 2) {
			const key = cellKey(x, y);
			if (createdCells.has(key)) continue; // already exists

			createdCells.add(key);

			for (const template of RHOMBUS_TEMPLATES) {
				createRhombus(template, x, y);
			}
		}
	}
}

function swapRhombus(state, el) {
	const fromSet = rhombusSets[state];
	const toSet = rhombusSets[1 - state];

	if (!el) {
		if (fromSet.size === 0) return; // nothing to swap
		const arr = Array.from(fromSet);
		el = arr[Math.floor(Math.random() * arr.length)];
		if (!el) return;
	}

	const next = 1 - state;

	el.style.opacity = next;
	el.dataset.state = next;

	fromSet.delete(el);

	setTimeout(() => {
		toSet.add(el);
	}, 1000);
}

function startSwapping() {
	if (swapTimer) clearInterval(swapTimer);

	swapTimer = setInterval(() => {
		if (window.scrollY > window.innerHeight + 24) return;

		if (rhombusSets[0].size > 0) swapRhombus(0);
		if (rhombusSets[1].size > 0) swapRhombus(1);
	}, 200);
}

function handleScroll() {
	const y = window.scrollY;
	rhombusContainer.style.position = "relative";
	rhombusContainer.style.overflow = "hidden";
	rhombusContainer.style.top = `${0.25 * y}px`;
	rhombusContainer.style.height = `${Math.max(20, window.innerHeight - 0.25 * y)}px`;
}

window.addEventListener("scroll", handleScroll);
window.addEventListener("resize", ensureGrid);

ensureGrid();
startSwapping();
handleScroll();