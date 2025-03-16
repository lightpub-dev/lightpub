import { connect, StringCodec } from 'nats';
import { mathjax } from 'mathjax-full/js/mathjax.js';
import { TeX } from 'mathjax-full/js/input/tex.js';
import { SVG } from 'mathjax-full/js/output/svg.js';
import { liteAdaptor } from 'mathjax-full/js/adaptors/liteAdaptor.js';
import { RegisterHTMLHandler } from 'mathjax-full/js/handlers/html.js';
import { AllPackages } from 'mathjax-full/js/input/tex/AllPackages.js';
import process from 'node:process'

// Configure MathJax
const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

const tex = new TeX({
  packages: AllPackages,
  inlineMath: [['$', '$'], ['\\(', '\\)']],
  displayMath: [['$$', '$$'], ['\\[', '\\]']],
  processEscapes: true,
  processEnvironments: true,
});

const svg = new SVG({ fontCache: 'none' });
const html = mathjax.document('', { InputJax: tex, OutputJax: svg });

// Function to render TeX to SVG
function renderMathJax(texContent: string, displayMode: boolean = false): string {
  try {
    const node = html.convert(texContent, {
      display: displayMode,
      em: 16,
      ex: 8,
      containerWidth: 800
    });
    
    return adaptor.innerHTML(node);
  } catch (error) {
    console.error('MathJax rendering error:', error);
    return `<span class="math-error">Failed to render TeX content</span>`;
  }
}

// Setup NATS connection
async function setupNats() {
  try {
    const nc = await connect({
      servers: process.env.NATS_URL || 'nats://localhost:4222',
    });
    
    console.log(`Connected to NATS at ${nc.getServer()}`);
    const sc = StringCodec();
    
    // Subscribe to the render request subject
    const subscription = nc.subscribe('lightpub.mathjax.render',{
      queue: 'mathjax-renderers'
    });
    console.log('Subscribed to lightpub.mathjax.render');
    
    // Process incoming render requests
    for await (const msg of subscription) {
      try {
        const requestData = JSON.parse(sc.decode(msg.data));
        const {  content, displayMode = false } = requestData;
        
        console.log(`Received render request ${content.substring(0, 10)}...`);
        
        const renderedSvg = renderMathJax(content, displayMode);
        
        // Reply with the rendered result
        const response = {
          success: true,
          result: renderedSvg
        };
        
        if (msg.reply) {
          nc.publish(msg.reply, sc.encode(JSON.stringify(response)));
          console.log(`Sent reply for request`);
        } else {
          console.warn(`No reply subject for request`);
        }
      } catch (error) {
        console.error('Error processing message:', error);
        if (msg.reply) {
          const response = {
            id: 'unknown',
            success: false,
            error: (error as any).message
          };
          nc.publish(msg.reply, sc.encode(JSON.stringify(response)));
        }
      }
    }
  } catch (error) {
    console.error('NATS connection error:', error);
    process.exit(1);
  }
}


// Connect to NATS
setupNats().catch(error => {
  console.error('Failed to start NATS client:', error);
  process.exit(1);
});