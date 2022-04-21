from sphinx.errors import ExtensionError

def add_ga_javascript(app, pagename, templatename, context, doctree):
    if not app.config.googleanalytics_id:
        return

    metatags = context.get('metatags', '')
    metatags += """<script type="text/javascript">

      var _gaq = _gaq || [];
      _gaq.push(['_setAccount', '%s']);
      _gaq.push(['_trackPageview']);

      (function() {
        var ga = document.createElement('script'); ga.type = 'text/javascript'; ga.async = true;
        ga.src = ('https:' == document.location.protocol ? 'https://ssl' : 'http://www') + '.google-analytics.com/ga.js';
        var s = document.getElementsByTagName('script')[0]; s.parentNode.insertBefore(ga, s);
      })();
    </script>""" % app.config.googleanalytics_id
    context['metatags'] = metatags

def setup(app):
    app.add_config_value('googleanalytics_id', '', 'html')
    app.connect('html-page-context', add_ga_javascript)
    return {'version': '0.1'}
