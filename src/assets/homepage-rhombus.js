let rhombusColors = ["cyan", "magenta", "yellow"];
let footerMessages = [];
{
	function nthWeekdayOfMonth(year, month, weekday, n) {
		// weekday : 0=sunday, 6=saturday
		// positive n : count from start of month
		// negative n : count from back of month
		// month is 1–12
		const first = new Date(year, month - 1, 1);
		const last  = new Date(year, month, 0);
		if (n > 0) {
			// nth from start
			const firstWeekday = first.getDay();
			const offset = (weekday - firstWeekday + 7) % 7;
			return 1 + offset + (n - 1) * 7;
		} else {
			// nth from end (-1 = last, -2 = second-to-last)
			const lastWeekday = last.getDay();
			const offset = (lastWeekday - weekday + 7) % 7;
			return last.getDate() - offset + (n + 1) * 7;
		}
	}

	
	function isThanksgiving(a=new Date){const b=a.getFullYear(),c=10,d=new Date(b,c,1),e=d.getDay();return a.getMonth()===c&&a.getDate()===1+(4-e+7)%7+21}

	const today = new Date();
	const currentYear = today.getFullYear();
	const currentMonth = today.getMonth() + 1; // 1–12
	const currentDay = today.getDate();        // 1–31

	function reg(month, day, colors, message) {
		if (currentMonth === month && currentDay === day) {
			if (colors) rhombusColors = colors;
			footerMessages.push(message);
		}
	}
	function regNth(month, weekday, n, colors, message) {
		const targetDay = nthWeekdayOfMonth(currentYear, month, weekday, n);
		if (currentMonth === month && currentDay === targetDay) {
			if (colors) rhombusColors = colors;
			footerMessages.push(message);
		}
	}

	// celebration is MANDATORY
	regNth(1, 1, 3, null, "Happy Martin Luther King Jr. Day!");
	reg(2, 14, ["#E53935", "#F48FB1", "#FFFFFF"], "Happy Valentine’s day!");
	regNth(2, 1, 3, null, "Happy Presidents’ Day!");
	reg(3, 8, null, "Happy International Women’s Day!");
	reg(3, 14, null, "Happy Pi Day!");
	reg(3, 17, ["#169B62", "#FF883E", "#FFFFFF"], "Happy Saint Patrick’s day!");
	reg(3, 31, ["#3BBEFA", "#F589D8", "#FFFFFF"], "Happy Trans Day of Visibility!");
	reg(4, 22, null, "Happy Earth Day!");
	reg(5, 1, null, "Happy International Workers’ Day!");
	if (currentMonth === 6) {
		footerMessages.push("Happy Pride Month!");
	}
	regNth(5, 0, 2, null, "Happy Mother’s Day!");
	regNth(5, 1, -1, null, "Happy Memorial Day!");
	reg(6, 19, null, "Happy Juneteenth!"); // ["#ce1126", "#fcd116", "#006b3f"]
	reg(6, 28, null, "Happy Tau Day! Tau rules pi drools!");
	reg(7, 4, ["#FFFFFF", "#224CE2", "#E52F29"], "Happy Independence Day!");
	reg(7, 14, null, "International Non‑Binary People’s Day!"); // im operating on levels of woke you can't even imagine
	reg(8, 9, null, "Happy World Indigenous Peoples’ Day!");
	regNth(9, 1, 1, null, "Happy Labor Day!");
	regNth(10, 1, 2, null, "Happy Indigenous Peoples’ Day!");
	reg(10, 31, ["#880000", "#006600", "#0000aa"], "Happy Halloween!");
	reg(11, 11, null, "Happy Veterans Day!");
	regNth(11, 4, 4, ["#d2691e", "#f5deb3", "#8b4513"], "Happy Thanksgiving!");
	reg(12, 10, null, "Happy Human Rights Day!");
	reg(12, 25, ["#FFFFFF", "#46903A", "#D93344"], "Merry Christmas!");
	
	if (currentMonth == 1 && currentDay == 1) footerMessages.push("Happy New Year’s "+currentYear+"!");
	if (currentMonth == 12 && currentDay == 31) footerMessages.push("Happy New Year’s Eve "+currentYear+"!");
	
	for (let [table, color, message] of [
		[
			{
				2026: [[3, 20]],
				2027: [[3, 10]],
				2028: [[2, 27]],
				2029: [[2, 15]],
				2030: [[2, 5]]
			}, null, "Eid Mubarak!" // eid al fitr
		],
		[
			{
				2026: [[5, 27]],
				2027: [[5, 17]],
				2028: [[5, 5]],
				2029: [[4, 24]],
				2030: [[4, 14]]
			}, null, "Eid Mubarak!" // eid al fitr
		],
		[
			{
				2026: [[4, 1]],
				2027: [[4, 21]],
				2028: [[4, 10]],
				2029: [[3, 30]],
				2030: [[4, 17]]
			}, null, "Happy Passover!" // or should i say chag pesach Big question
		],
		[
			{
				2026: [[9, 12]],
				2027: [[10, 2]],
				2028: [[9, 20]],
				2029: [[9, 9]],
				2030: [[9, 28]]
			}, null, "Shanah Tovah!" // rosh hashanah
		],
		[
			{
				2026: [[9, 21]],
				2027: [[10, 11]],
				2028: [[9, 30]],
				2029: [[9, 18]],
				2030: [[10, 7]]
			}, null, "G'mar Chatima Tova!" // yom kippur
		],
		[
			{
				2026: [[12, 8]],
				2027: [[11, 28]],
				2028: [[12, 16]],
				2029: [[12, 5]],
				2030: [[12, 25]]
			}, null, "Happy Hanukkah!"
		],
		[
			{
				2026: [[4, 5]],
				2027: [[3, 28]],
				2028: [[4, 16]],
				2029: [[4, 1]],
				2030: [[4, 21]]
			}, null, "Happy Easter!"
		],
		[
			{
				2026: [[6, 21]],
				2027: [[6, 21]],
				2028: [[6, 20]],
				2029: [[6, 20]],
				2030: [[6, 21]]
			}, null, "Happy summer solstice!"
		],
		[
			{
				2026: [[12, 21]],
				2027: [[12, 21]],
				2028: [[12, 21]],
				2029: [[12, 21]],
				2030: [[12, 21]]
			}, null, "Happy winter solstice!"
		],
		[
			{
				2026: [[3, 20]],
				2027: [[3, 20]],
				2028: [[3, 20]],
				2029: [[3, 20]],
				2030: [[3, 20]]
			}, null, "Happy spring equinox!"
		],
		[
			{
				2026: [[9, 22]],
				2027: [[9, 22]],
				2028: [[9, 22]],
				2029: [[9, 22]],
				2030: [[9, 22]]
			}, null, "Happy fall equinox!"
		],
	]) {
		if (table[currentYear]) for (let pair of table[currentYear]) {
			reg(...pair, color, message);
		}
	}
	
	// chinese new year
	{
		// Xu's algorithm from the 90s. valid until 2100
		const C = [
			0, 31, 21, 10, 30, 18, 6, 26, 14, 3, 23, 11,
			1, 29, 17, 5, 25, 13, 2, 22, 9, 28, 16, 4,
			24, 12, 1, 20, 8, 27, 15, 3, 23, 11, 31, 19,
			7, 26, 14, 4, 24, 12, 1, 21, 9, 29, 17, 6
		];
		const idx = (currentYear - 1901) % 60;
		const day = C[idx];
		const month = day > 20 ? 1 : 2;
		reg(month, day, null, "Happy Lunar New Year!")
	}
	
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

	//const state = Math.random() < 0.1 ? 1 : 0;
	const state = 0;
	el.dataset.state = state;

	rhombusSets[state].add(el);

	el.addEventListener("click", () => {
		swapRhombus(Number(el.dataset.state), el);
		console.log(el);
	});

	rhombusContainer.appendChild(el);
	
	if (Math.random() < 0.1) {
		setTimeout(function() {
			swapRhombus(0, el);
		}, Math.random() * 2000)
	}
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

const footerPolygons = Array.from(document.querySelectorAll("footer polygon"));
for (let i = 0; i < 3; i++) footerPolygons[i].style.fill = rhombusColors[i];

for (let footerMessage of footerMessages.sort()) {
	const div = document.querySelector("footer").insertBefore(document.createElement("div"), document.querySelector("footer svg"));
	div.innerText = footerMessage;
}