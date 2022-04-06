from typing import Dict
from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

app = FastAPI()

app.mount("/static", StaticFiles(directory="static"), name="static")


templates = Jinja2Templates(directory="templates")




@app.get("/search/")
async def search():
    print("I am invoked")
    return  [{"value": "ankur"}, {"value":"ana"}, {"value": "mimi"}, {"value":"vikki"}]

@app.get("/items/{id}", response_class=HTMLResponse)
async def read_item(request: Request, id: str):
    for route in app.routes:
        print(route.name, route.path)
    # print(request.url_for("search", path_params={"query": "2"}))
    
    return templates.TemplateResponse("search.html", {"request": request, "id": id})
