/* let playSyncedButton = document.querySelector("form").appendChild(document.createElement("a"));
playSyncedButton.innerText = "Show lyrics in sync with audio";
playSyncedButton.style.marginLeft = "2rem"; //scuffed

playSyncedButton.onclick = function() {
	const audio = new Audio(source);
	audio.preservesPitch = false;
	audio.playbackRate = 1;
	window.nightcore = function(x) {
		audio.playbackRate = x;
	};
	audio.play().then(() => {
		// success path yay audio exists and works Joy to the world
		this.remove();
		document.body.setAttribute("data-playing", "true");
		const aheadTime = 0.1; // one half the transition animation time
		const moments = Array.from(document.querySelectorAll("l-l")).map(function(ll) {
			return {
				element: ll,
				start: parseFloat(ll.getAttribute("data-start")) - aheadTime,
				end: parseFloat(ll.getAttribute("data-end")) - aheadTime
			};
		});
		let currentMoment = null; // Option<Moment>
		
		let apContainer = document.querySelector("form").appendChild(document.createElement("ap-container"));
		let apPlayback = apContainer.appendChild(document.createElement("ap-playback"));
		let apBar = apContainer.appendChild(document.createElement("ap-bar"));
		let apProgress = apBar.appendChild(document.createElement("ap-progress"));
		let apTimestamp = apContainer.appendChild(document.createElement("ap-timestamp"));
		apTimestamp.textContent = "0:00";
		
		let drag = false;
		audio.ontimeupdate = () => {
			if (!drag && audio.duration)
				apProgress.style.width = (audio.currentTime / audio.duration * 100) + "%";
			const seconds = Math.round(audio.currentTime);
			const m = Math.floor(seconds / 60);
			const s = seconds % 60;
			apTimestamp.textContent = m + ":" + (s < 10 ? "0" + s : s);
		};
		
		function seek(e) {
			const r = apBar.getBoundingClientRect();
			const start = r.left;
			const end = r.right;
			let x = Math.max(start, Math.min(e.clientX, end));
			let ratio = (x - start) / (end - start);
			apProgress.style.width = (ratio * 100) + "%";
			if (audio.duration) audio.currentTime = ratio * audio.duration;
			audio.ontimeupdate();
		}
		
		setInterval(function() {
			if (audio.paused) return;
			if (currentMoment) {
				// check to see if we should escape current moment
				if (audio.currentTime < currentMoment.start || audio.currentTime >= currentMoment.end) {
					findNewMoment();
				}
			} else {
				findNewMoment(); //bleh is there some way to make this faster
			}
		}, 50);
		
		function findNewMoment() {
			const oldMoment = currentMoment;
			let newMoment = null;
			for (let moment of moments) {
				if (audio.currentTime >= moment.start && audio.currentTime < moment.end) {
					newMoment = moment;
				}
			}
			if (oldMoment) oldMoment.element.setAttribute("data-current", "false");
			currentMoment = newMoment;
			if (newMoment) newMoment.element.setAttribute("data-current", "true");
		}
		
		apPlayback.onmouseup = e => {
			//e.stopPropagation();
			if (audio.paused) {
				audio.play();
				document.body.setAttribute("data-playing", "true");
			} else {
				audio.pause();
				document.body.setAttribute("data-playing", "false");
			}
		};

		apBar.onmousedown = e => {
			//const r = apBar.getBoundingClientRect();
			//if (e.clientX < r.left + apPlayback.offsetWidth) return;
			drag = true;
			seek(e);
		};

		document.onmousemove = e => drag && seek(e);
		document.onmouseup = () => drag = false;
	}, () => {
		// failure path
		this.remove();
		let errorMessage = document.querySelector("form").appendChild(document.createElement("error-message"));
		errorMessage.innerText = "Failed to load audio";
		errorMessage.style.marginLeft = "2rem"; //scuffed
	});
}