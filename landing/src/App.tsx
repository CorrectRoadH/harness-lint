import {
  commands,
  features,
  footerLinks,
  GITHUB_URL,
  hero,
  heroTerminal,
  install,
  problem,
  steps,
} from "./content";
import { Terminal } from "./components/Terminal";
import { CodeBlock } from "./components/CodeBlock";

export default function App() {
  return (
    <>
      <Nav />
      <main>
        <Hero />
        <Problem />
        <HowItWorks />
        <Features />
        <Commands />
        <Install />
      </main>
      <Footer />
    </>
  );
}

function Nav() {
  return (
    <header className="nav">
      <div className="container nav-inner">
        <a className="brand" href="#top">
          <span className="brand-mark">▦</span>
          <span>harness-lint</span>
        </a>
        <nav className="nav-links">
          <a href="#how">How it works</a>
          <a href="#features">Features</a>
          <a href="#install">Install</a>
          <a className="nav-gh" href={GITHUB_URL}>
            GitHub ↗
          </a>
        </nav>
      </div>
    </header>
  );
}

function Hero() {
  return (
    <section className="hero" id="top">
      <div className="container hero-grid">
        <div className="hero-copy">
          <span className="eyebrow">{hero.eyebrow}</span>
          <h1>{hero.title}</h1>
          <p className="lead">{hero.subtitle}</p>
          <div className="cta-row">
            <a className="btn btn-primary" href={hero.primaryCta.href}>
              {hero.primaryCta.label}
            </a>
            <a className="btn btn-ghost" href={hero.secondaryCta.href}>
              {hero.secondaryCta.label}
            </a>
          </div>
        </div>
        <div className="hero-visual">
          <Terminal title="zsh — harness-lint" lines={heroTerminal} />
        </div>
      </div>
    </section>
  );
}

function Problem() {
  return (
    <section className="band">
      <div className="container narrow">
        <h2 className="band-title">{problem.title}</h2>
        <p className="band-body">{problem.body}</p>
      </div>
    </section>
  );
}

function HowItWorks() {
  return (
    <section className="section" id="how">
      <div className="container">
        <SectionHead
          kicker="The loop"
          title="One correction. Enforced forever."
        />
        <ol className="steps">
          {steps.map((s) => (
            <li className="step" key={s.n}>
              <span className="step-n">{s.n}</span>
              <h3>{s.title}</h3>
              <p>{s.body}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}

function Features() {
  return (
    <section className="section" id="features">
      <div className="container">
        <SectionHead
          kicker="Why harness-lint"
          title="Linting designed for agents, not just humans."
        />
        <div className="feature-grid">
          {features.map((f) => (
            <article className="card" key={f.title}>
              <h3>{f.title}</h3>
              <p>{f.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

function Commands() {
  return (
    <section className="section" id="commands">
      <div className="container">
        <SectionHead
          kicker="The CLI"
          title="A handful of commands do the work."
        />
        <ul className="cmd-list">
          {commands.map((c) => (
            <li className="cmd-row" key={c.cmd}>
              <code>{c.cmd}</code>
              <span className="cmd-note">{c.note}</span>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}

function Install() {
  return (
    <section className="section install" id="install">
      <div className="container narrow">
        <SectionHead kicker="Get started" title="Up and running in three steps." />
        <div className="install-steps">
          {install.map((step) => (
            <div className="install-step" key={step.label}>
              <span className="install-label">{step.label}</span>
              <CodeBlock code={step.code} />
              {step.note && <p className="install-note">{step.note}</p>}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function SectionHead({ kicker, title }: { kicker: string; title: string }) {
  return (
    <div className="section-head">
      <span className="kicker">{kicker}</span>
      <h2>{title}</h2>
    </div>
  );
}

function Footer() {
  return (
    <footer className="footer">
      <div className="container footer-inner">
        <div className="footer-brand">
          <span className="brand-mark">▦</span>
          <span>harness-lint</span>
          <p>Lint Driven Development for coding agents.</p>
        </div>
        <nav className="footer-links">
          {footerLinks.map((l) => (
            <a key={l.href} href={l.href}>
              {l.label} ↗
            </a>
          ))}
        </nav>
      </div>
      <div className="container footer-fine">
        <span>MIT licensed · Built with Rust &amp; GritQL.</span>
      </div>
    </footer>
  );
}
