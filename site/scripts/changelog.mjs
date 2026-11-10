// The changelog page (/changelog): every commit on main, newest first and grouped by day, read
// from git at build time like the home page's commit count. After Wayne Sutton's profile page
// changelog in his fork (waynesutton/threadbear, web/src/profile-changelog.ts): the subject's
// "Area:" prefix becomes a chip and the short sha links to the commit on GitHub.
//
// It lists what's been published, not what's on this Mac: commits land on the local main first
// and reach GitHub later (the release publisher pushes them), so the build reads origin/main,
// which only ever holds what's on GitHub, and every sha link already works. The site's only
// deployed by hand, so site.js tops the page up from GitHub's public API with what's on main
// that the build's commit (data-built) hasn't got.
import { execFileSync } from "node:child_process";

/** Days are Mike's, in Perth, the same as the home page's first day. */
export const TIME_ZONE = "Australia/Perth";

/**
 * "Profile page: its addresses as two rows" is area "Profile page". Only a short, plain prefix
 * counts, so a sentence with a colon in it or a URL stays whole. Kept the same as site.js's.
 */
export function splitSubject(subject) {
  const s = subject.trim();
  const m = /^([^:()[\]`{}]{2,32}):\s+(\S.*)$/.exec(s);
  if (!m || /https?$/i.test(m[1])) return { text: s };
  return { area: m[1].trim(), text: m[2].charAt(0).toUpperCase() + m[2].slice(1) };
}

/** YYYY-MM-DD in Perth for an ISO time. */
export function perthDay(iso) {
  return new Intl.DateTimeFormat("en-CA", { timeZone: TIME_ZONE, year: "numeric", month: "2-digit", day: "2-digit" }).format(new Date(iso));
}

/** "Monday" and "9 November 2026" for a YYYY-MM-DD (a phone hides the weekday, to fit one line). */
export function longDay(day) {
  const d = new Date(`${day}T12:00:00Z`);
  return {
    weekday: d.toLocaleDateString("en-AU", { weekday: "long", timeZone: "UTC" }),
    date: d.toLocaleDateString("en-AU", { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" }),
  };
}

/**
 * The commits on `ref` as { sha, date, area?, text }, newest first, merges left out (a fork's
 * "Bring in" merges and the odd PR merge say nothing about what changed). The date is when it
 * landed on main (committer date), not when it was first written, since landing rebases.
 */
export function readCommits(root, ref) {
  const out = execFileSync("git", ["-C", root, "log", ref, "--no-merges", "--format=%H%x1f%cI%x1f%s%x1e"], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  return out.split("\x1e").map((r) => r.trim()).filter(Boolean).map((r) => {
    const [sha, date, subject] = r.split("\x1f");
    return { sha, date, ...splitSubject(subject ?? "") };
  }).filter((c) => /^[0-9a-f]{40}$/.test(c.sha) && c.text);
}

/**
 * What's on GitHub: origin/main as of the last fetch, else the newest release/* tag, else HEAD
 * (a copy with neither, where the sha links may not work yet but the page still builds).
 */
export function publishedRef(root) {
  const git = (...args) => execFileSync("git", ["-C", root, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  try {
    return git("rev-parse", "--verify", "--quiet", "refs/remotes/origin/main") && "refs/remotes/origin/main";
  } catch {
    const tag = git("for-each-ref", "--sort=-creatordate", "--count=1", "--format=%(refname:short)", "refs/tags/release/*");
    return tag || "HEAD";
  }
}

/** The full sha `ref` points at: what site.js asks GitHub to compare main with. */
export function refSha(root, ref) {
  return execFileSync("git", ["-C", root, "rev-parse", `${ref}^{commit}`], { encoding: "utf8" }).trim();
}

/** Commits gathered under their Perth day, newest day first, in the order given within a day. */
export function byDay(commits) {
  const days = new Map();
  for (const c of commits) {
    const day = perthDay(c.date);
    if (!days.has(day)) days.set(day, []);
    days.get(day).push(c);
  }
  return [...days.entries()].sort(([a], [b]) => (a < b ? 1 : -1)).map(([day, list]) => ({ day, commits: list }));
}

/**
 * Commit text for the page. HTML-escaped, and also: em and en dashes become a comma and a hyphen
 * (the build refuses a page with either), and braces become entities so a subject that mentions
 * a {{placeholder}} doesn't trip the build's leftover-placeholder check.
 */
export function pageText(s) {
  return s
    .replace(/\s*—\s*/g, ", ").replace(/–/g, "-")
    .replace(/[&<>"{}]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "{": "&#123;", "}": "&#125;" })[c]);
}

const plural = (n) => `${n.toLocaleString("en-AU")} change${n === 1 ? "" : "s"}`;

/** One commit's row. site.js builds the same markup for the ones it adds. */
function row(c, githubUrl) {
  const area = c.area ? `<span class="cl-area">${pageText(c.area)}</span>` : "";
  return `            <li>${area}<span class="cl-text">${pageText(c.text)}</span><a class="cl-sha" href="${githubUrl}/commit/${c.sha}" aria-label="Commit ${c.sha.slice(0, 7)} on GitHub">${c.sha.slice(0, 7)}</a></li>`;
}

/** The page's content, for writePage. */
export function changelogPage({ commits, built, githubUrl, repo }) {
  const days = byDay(commits);
  const sections = days.map(({ day, commits: list }) => `        <section class="cl-day" id="${day}" aria-labelledby="d-${day}">
          <h2 id="d-${day}"><a href="#${day}"><time datetime="${day}"><span class="cl-weekday">${longDay(day).weekday} </span>${longDay(day).date}</time></a><span class="cl-count">${plural(list.length)}</span></h2>
          <ul>
${list.map((c) => row(c, githubUrl)).join("\n")}
          </ul>
        </section>`);
  return `    <article class="legal changelog" data-repo="${repo}" data-built="${built}" data-commit-url="${githubUrl}/commit/">
      <h1>Changelog</h1>
      <p class="lede">Everything that's gone into Threadbear, newest first. Each line is a commit on <a href="${githubUrl}/commits/main">main</a>, and forks get them as a daily release.</p>
      <div class="cl-days">
${sections.join("\n")}
      </div>
    </article>`;
}
