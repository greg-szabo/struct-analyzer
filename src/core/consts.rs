pub const HEADER: &str = r#"## Tendermint public JSON-serializable structures - draw.io CSV export
# label: %name%
# stylename: color
# styles: { \
#            "red": "shape=%shape%;rounded=1;fillColor=#f8cecc;strokeColor=#b85450;strokeWidth=2",\
#            "green": "shape=%shape%;rounded=1;fillColor=#d5e8d4;strokeColor=#82b366;strokeWidth=2",\
#            "blue": "shape=%shape%;rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;strokeWidth=2",\
#            "yellow": "shape=%shape%;rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;strokeWidth=2",\
#            "white": "shape=%shape%;rounded=1;fillColor=#ffffff;strokeColor=#000000;strokeWidth=2",\
#            "green_gradient": "shape=%shape%;rounded=1;fillColor=#d5e8d4;strokeColor=#82b366;strokeWidth=2;gradientColor=#ffffff",\
#            "blue_gradient": "shape=%shape%;rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;strokeWidth=2;gradientColor=#ffffff",\
#            "yellow_gradient": "shape=%shape%;rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;strokeWidth=2;gradientColor=#ffffff",\
#            "legend": "shape=%shape%;rounded=1;shadow=1;fontSize=16;align=left;whiteSpace=wrap;html=1;fillColor=#d0cee2;strokeWidth=2;strokeColor=#56517e;"\
# }
# connect: {"from":"refs", "to":"name", "invert":false, "style":"curved=1;endArrow=blockThin;endFill=1;"}
# connect: {"from":"refs2", "to":"name", "invert":false, "style":"curved=1;endArrow=blockThin;endFill=1;dashed=1;dashPattern=1 4;strokeColor=none;"}
# namespace: tendermint-
# width: auto
# height: auto
# padding: 10
# ignore: refs,refs2
# nodespacing: 60
# levelspacing: 60
# edgespacing: 60
# layout: horizontalflow
name,shape,color,refs,refs2
"<b>LEGEND<br><br><b style=\"color:#d5e8d4;\">Green:</b> #[derive(Deserialize, Serialize)]<br><b style=\"color:#dae8fc;\">Blue:</b> #[serde(try_from = \"\", into = \"\")]<br><b style=\"color:#fff2cc;\">Yellow:</b> impl Deserialize/Serialize for my_struct {}<br><b style=\"color:#ffffff;\">White:</b> No serialization<br><br>Gradient color: asymmetric serialization<br>Red: invalid combination of features<br>Rounded rectangle: struct<br>Ellipse: enum</b>",rectangle,legend,"#;
